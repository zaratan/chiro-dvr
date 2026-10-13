#[path = "support/sequential.rs"]
mod sequential;

use std::num::NonZeroU32;
use std::path::Path;

use media::convert::{Converter, SourceRange};
use media::decode::{Decoder, Threads};
use media::encode::Codec;
use media::encode::fallback::Fallback;
use media::frame::{ColorRange, YuvFrame};
use reference::location::committed;
use reference::recording::Recording;
use render::clips::{Clip, Outputs, render};
use render::config::RenderConfig;
use render::source::{Error, Source};
use render::windows::{Window, clip_windows};
use types::detection::Detection;
use types::frame::{FrameIndex, Pts};
use types::span::Span;
use types::track::Track;

use crate::sequential::Sequential;

const EXTRACT: &str = "video_092_original_3m24-4m05";
const LENGTHS: [usize; 4] = [380, 108, 115, 117];
const BLOCK: usize = 8;
const C1: f64 = 6.5025;
const C2: f64 = 58.5225;
const LABEL_ROOM: f64 = 48.0;
const MARGIN: f64 = 8.0;
const ZOOM_THRESHOLD: f64 = 0.953;

#[derive(Debug, Default, Clone, Copy)]
struct Similarity {
    sum: f64,
    blocks: usize,
}

impl Similarity {
    fn mean(self) -> f64 {
        self.sum / self.blocks as f64
    }
}

#[derive(Debug, Clone, Copy)]
struct Area {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl Area {
    fn touches(&self, x: usize, y: usize) -> bool {
        let (left, top) = (x as f64, y as f64);
        let (right, bottom) = (left + BLOCK as f64, top + BLOCK as f64);
        left < self.right && self.left < right && top < self.bottom && self.top < bottom
    }
}

#[cfg(test)]
fn block_similarity(first: &[u8], second: &[u8], width: usize, x: usize, y: usize) -> f64 {
    let pixels = (BLOCK * BLOCK) as f64;
    let rows = (y..y + BLOCK).flat_map(|row| (x..x + BLOCK).map(move |column| row * width + column));
    let pairs: Vec<(f64, f64)> = rows
        .map(|index| (f64::from(first[index]), f64::from(second[index])))
        .collect();
    let mean_a = pairs.iter().map(|pair| pair.0).sum::<f64>() / pixels;
    let mean_b = pairs.iter().map(|pair| pair.1).sum::<f64>() / pixels;
    let (mut var_a, mut var_b, mut covariance) = (0.0, 0.0, 0.0);
    for (a, b) in &pairs {
        var_a += (a - mean_a) * (a - mean_a);
        var_b += (b - mean_b) * (b - mean_b);
        covariance += (a - mean_a) * (b - mean_b);
    }
    let (var_a, var_b, covariance) = (var_a / pixels, var_b / pixels, covariance / pixels);
    ((2.0 * mean_a * mean_b + C1) * (2.0 * covariance + C2))
        / ((mean_a * mean_a + mean_b * mean_b + C1) * (var_a + var_b + C2))
}

#[cfg(test)]
fn masked_similarity(first: &[u8], second: &[u8], width: usize, height: usize, masked: &[Area]) -> Similarity {
    let mut similarity = Similarity::default();
    for y in (0..=height - BLOCK).step_by(BLOCK) {
        for x in (0..=width - BLOCK).step_by(BLOCK) {
            if masked.iter().any(|area| area.touches(x, y)) {
                continue;
            }
            similarity.sum += block_similarity(first, second, width, x, y);
            similarity.blocks += 1;
        }
    }
    similarity
}

#[cfg(test)]
fn drawn_areas(tracks: &[Track], frame: usize, trail_frames: usize, pad: f64) -> Vec<Area> {
    tracks
        .iter()
        .filter(|track| (track.first().frame.get()..=track.last().frame.get()).contains(&frame))
        .map(|track| {
            let points = track.points();
            let next = points.partition_point(|point| point.frame.get() < frame);
            let previous = points.partition_point(|point| point.frame.get() + trail_frames + 1 < frame);
            points[previous.saturating_sub(1)..=next.min(points.len() - 1)]
                .iter()
                .map(|point| Area {
                    left: point.left - pad - MARGIN,
                    top: point.top - pad - LABEL_ROOM - MARGIN,
                    right: point.left + point.width + pad + LABEL_ROOM + MARGIN,
                    bottom: point.top + point.height + pad + MARGIN,
                })
                .reduce(|hull, area| Area {
                    left: hull.left.min(area.left),
                    top: hull.top.min(area.top),
                    right: hull.right.max(area.right),
                    bottom: hull.bottom.max(area.bottom),
                })
                .unwrap()
        })
        .collect()
}

#[cfg(test)]
fn rgb_at(converter: &mut Converter, frame: &YuvFrame, x: usize, y: usize) -> [u8; 3] {
    let mut rgba = vec![0; frame.width() as usize * frame.height() as usize * 4];
    converter.yuv_to_rgba(frame, &mut rgba).unwrap();
    let index = (y * frame.width() as usize + x) * 4;
    [rgba[index], rgba[index + 1], rgba[index + 2]]
}

#[cfg(test)]
struct Comparison<'a> {
    source: &'a Source,
    read: &'a Sequential,
    tracks: &'a [Track],
    trail_frames: usize,
    pad: f64,
}

#[cfg(test)]
fn compare_clip(comparison: &Comparison<'_>, window: &Window, clip: &Path) -> Similarity {
    let mut decoder = Decoder::open(clip, Threads::Automatic).unwrap();
    let mut written = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut round_trip = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut display: Option<(Converter, Vec<u8>)> = None;
    let mut similarity = Similarity::default();
    comparison
        .source
        .serve(window.span, |frame, pixels| {
            assert_eq!(
                sequential::fingerprint(pixels),
                comparison.read.fingerprints[frame.get()]
            );
            assert!(decoder.next(&mut written).unwrap().is_some(), "{frame:?}");
            let (width, height) = (pixels.width(), pixels.height());
            let (converter, rgba) = display.get_or_insert_with(|| {
                let converter = Converter::new(width, height, SourceRange::FullBlackAt16).unwrap();
                (converter, vec![0; width as usize * height as usize * 4])
            });
            converter.yuv_to_rgba(pixels, rgba).unwrap();
            converter.rgba_to_yuv(rgba, &mut round_trip).unwrap();
            let masked = drawn_areas(comparison.tracks, frame.get(), comparison.trail_frames, comparison.pad);
            let one = masked_similarity(round_trip.y(), written.y(), width as usize, height as usize, &masked);
            similarity.sum += one.sum;
            similarity.blocks += one.blocks;
            Ok::<(), Error>(())
        })
        .unwrap();
    assert!(decoder.next(&mut written).unwrap().is_none(), "{}", clip.display());
    similarity
}

#[cfg(test)]
fn report_colours(track: &Track, window: &Window, clip: &Path, pad: f64) {
    let point = track.points()[track.points().len() / 2];
    let mut decoder = Decoder::open(clip, Threads::Automatic).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    for _ in window.span.first().get()..=point.frame.get() {
        decoder.next(&mut frame).unwrap();
    }
    let mut converter = Converter::new(frame.width(), frame.height(), SourceRange::Limited).unwrap();
    let edge = ((point.left + point.width + pad) as usize, point.y as usize + 4);
    let trail = (point.x as usize, point.y as usize);
    println!(
        "{}: box {:?}, trail {:?}",
        clip.display(),
        rgb_at(&mut converter, &frame, edge.0, edge.1),
        rgb_at(&mut converter, &frame, trail.0, trail.1)
    );
}

#[test]
#[ignore = "slow"]
fn the_similarity_is_one_for_the_same_image_and_known_for_an_offset() {
    let width = 16;
    let first: Vec<u8> = (0..width * BLOCK).map(|index| (index * 7 % 200) as u8).collect();
    let same = masked_similarity(&first, &first, width, BLOCK, &[]);
    assert_eq!((same.mean(), same.blocks), (1.0, 2));
    let flat = vec![100; width * BLOCK];
    let raised = vec![110; width * BLOCK];
    let offset = masked_similarity(&flat, &raised, width, BLOCK, &[]);
    assert!((offset.mean() - 0.995_476_444_091_506_6).abs() < 1e-12, "{offset:?}");
    let hidden = Area {
        left: 9.0,
        top: 0.0,
        right: 10.0,
        bottom: 1.0,
    };
    let masked = masked_similarity(&flat, &raised, width, BLOCK, &[hidden]);
    assert_eq!(masked.blocks, 1);
}

#[test]
#[ignore = "slow"]
fn outside_the_drawings_each_clip_keeps_the_source_frames_of_the_same_pts() {
    let path = reference::fixture::fixture(&format!("{EXTRACT}.mp4")).unwrap();
    let read = sequential::read(&path);
    let tracks = Recording::read(&committed(EXTRACT)).unwrap().tracks;
    let source = Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap();
    let rate = read.timeline.rate();
    let config = RenderConfig::default();
    let windows = clip_windows(&tracks, read.timeline.frame_count(), config, rate);
    let lengths: Vec<usize> = windows.iter().map(|window| window.span.frame_count()).collect();
    assert_eq!(lengths, LENGTHS);
    let comparison = Comparison {
        source: &source,
        read: &read,
        tracks: &tracks,
        trail_frames: (config.trail_s() * rate.get().to_f64()).round() as usize,
        pad: f64::from(config.box_pad()),
    };
    let decoder = Decoder::open(&path, Threads::Automatic).unwrap();
    let stream = decoder.stream();
    let available = Fallback::for_this_platform()
        .trial(stream.width(), stream.height(), stream.frame_rate().get())
        .unwrap();
    for codec in available.codecs() {
        let folder = tempfile::tempdir().unwrap();
        let clips: Vec<Clip> = (0..tracks.len())
            .map(|track| Clip {
                track,
                path: folder.path().join(format!("clip_{track}.mp4")),
            })
            .collect();
        let outputs = Outputs {
            clips: clips.clone(),
            zooms: Vec::new(),
            annotated: None,
        };
        render(&source, &tracks, &outputs, config, &[codec]).unwrap();
        for ((window, clip), track) in windows.iter().zip(&clips).zip(&tracks) {
            let similarity = compare_clip(&comparison, window, &clip.path);
            println!(
                "{codec:?} {}: {:.4} on {} blocks",
                clip.path.display(),
                similarity.mean(),
                similarity.blocks
            );
            report_colours(track, window, &clip.path, comparison.pad);
            assert!(similarity.mean() >= threshold(codec), "{codec:?}: {similarity:?}");
        }
    }
}

#[cfg(test)]
fn threshold(codec: Codec) -> f64 {
    match codec {
        Codec::VideoToolbox => 0.963,
        Codec::OpenH264 => 0.964,
    }
}

#[cfg(test)]
fn gcd(a: usize, b: usize) -> usize {
    if b == 0 { a } else { gcd(b, a % b) }
}

#[cfg(test)]
fn median(mut values: Vec<f64>) -> f64 {
    values.sort_by(f64::total_cmp);
    let middle = values.len() / 2;
    if values.len() % 2 == 1 {
        values[middle]
    } else {
        f64::midpoint(values[middle - 1], values[middle])
    }
}

#[cfg(test)]
fn expected_frame(track: &Track, width: usize, height: usize) -> (usize, usize, usize, usize) {
    let points = track.points();
    let widest = 4.0 * median(points.iter().map(|point| point.width).collect());
    let tallest = 4.0 * median(points.iter().map(|point| point.height).collect());
    let boxes: Vec<(f64, f64, f64, f64)> = points
        .iter()
        .map(|point| {
            let (w, h) = (point.width.min(widest), point.height.min(tallest));
            let left = if point.width > widest {
                point.x - w / 2.0
            } else {
                point.left
            };
            let top = if point.height > tallest {
                point.y - h / 2.0
            } else {
                point.top
            };
            (left, top, left + w, top + h)
        })
        .collect();
    let x0 = boxes.iter().map(|b| b.0).fold(f64::INFINITY, f64::min) - 40.0;
    let y0 = boxes.iter().map(|b| b.1).fold(f64::INFINITY, f64::min) - 40.0;
    let x1 = boxes.iter().map(|b| b.2).fold(f64::NEG_INFINITY, f64::max) + 40.0;
    let y1 = boxes.iter().map(|b| b.3).fold(f64::NEG_INFINITY, f64::max) + 40.0;
    let units = gcd(width, height);
    let (unit_w, unit_h) = (width / units, height / units);
    let count = (((x1 - x0) / unit_w as f64).ceil() as usize)
        .max(((y1 - y0) / unit_h as f64).ceil() as usize)
        .clamp(units.div_ceil(4), units);
    let (crop_w, crop_h) = (count * unit_w, count * unit_h);
    let place = |center: f64, size: usize, limit: usize| {
        ((center - size as f64 / 2.0).round().max(0.0) as usize).min(limit - size)
    };
    (
        place(f64::midpoint(x0, x1), crop_w, width),
        place(f64::midpoint(y0, y1), crop_h, height),
        crop_w,
        crop_h,
    )
}

#[cfg(test)]
fn compare_zoom(comparison: &Comparison<'_>, window: &Window, track: &Track, zoom: &Path) -> Similarity {
    let mut decoder = Decoder::open(zoom, Threads::Automatic).unwrap();
    let mut written = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut round_trip = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut display: Option<(Converter, Vec<u8>)> = None;
    let mut similarity = Similarity::default();
    comparison
        .source
        .serve(window.span, |frame, pixels| {
            assert!(decoder.next(&mut written).unwrap().is_some(), "{frame:?}");
            let (width, height) = (pixels.width() as usize, pixels.height() as usize);
            let (converter, rgba) = display.get_or_insert_with(|| {
                let converter = Converter::new(pixels.width(), pixels.height(), SourceRange::FullBlackAt16).unwrap();
                (converter, vec![0; width * height * 4])
            });
            converter.yuv_to_rgba(pixels, rgba).unwrap();
            converter.rgba_to_yuv(rgba, &mut round_trip).unwrap();
            let (left, top, crop_w, crop_h) = expected_frame(track, width, height);
            let mut expected = vec![0; width * height];
            for y in 0..height {
                for x in 0..width {
                    let (source_x, source_y) = (left + x * crop_w / width, top + y * crop_h / height);
                    expected[y * width + x] = round_trip.y()[source_y * width + source_x];
                }
            }
            let scale = width as f64 / crop_w as f64;
            let reach = 16.0 * scale + 16.0;
            let near: Vec<&Detection> = track
                .points()
                .iter()
                .filter(|point| {
                    point.frame.get() + comparison.trail_frames + 1 >= frame.get()
                        && point.frame.get() <= frame.get() + 1
                })
                .collect();
            let masked: Vec<Area> = near
                .iter()
                .map(|point| Area {
                    left: (point.left - left as f64) * scale - reach,
                    top: (point.top - top as f64) * scale - reach,
                    right: (point.left + point.width - left as f64) * scale + reach,
                    bottom: (point.top + point.height - top as f64) * scale + reach,
                })
                .reduce(|hull, area| Area {
                    left: hull.left.min(area.left),
                    top: hull.top.min(area.top),
                    right: hull.right.max(area.right),
                    bottom: hull.bottom.max(area.bottom),
                })
                .into_iter()
                .collect();
            let one = masked_similarity(&expected, written.y(), width, height, &masked);
            similarity.sum += one.sum;
            similarity.blocks += one.blocks;
            Ok::<(), Error>(())
        })
        .unwrap();
    assert!(decoder.next(&mut written).unwrap().is_none(), "{}", zoom.display());
    similarity
}

#[test]
#[ignore = "slow"]
fn outside_the_trail_each_zoom_is_the_nearest_magnification_of_its_frame_at_a_quarter_of_the_rate() {
    let path = reference::fixture::fixture(&format!("{EXTRACT}.mp4")).unwrap();
    let read = sequential::read(&path);
    let tracks = Recording::read(&committed(EXTRACT)).unwrap().tracks;
    let source = Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap();
    let rate = read.timeline.rate();
    let config = RenderConfig::default();
    let windows = clip_windows(&tracks, read.timeline.frame_count(), config, rate);
    let comparison = Comparison {
        source: &source,
        read: &read,
        tracks: &tracks,
        trail_frames: (config.trail_s() * rate.get().to_f64()).round() as usize,
        pad: f64::from(config.box_pad()),
    };
    let folder = tempfile::tempdir().unwrap();
    let zooms: Vec<Clip> = (0..tracks.len())
        .map(|track| Clip {
            track,
            path: folder.path().join(format!("clip_{track}_zoom.mp4")),
        })
        .collect();
    let outputs = Outputs {
        clips: Vec::new(),
        zooms: zooms.clone(),
        annotated: None,
    };
    render(
        &source,
        &tracks,
        &outputs,
        config,
        &[Codec::VideoToolbox, Codec::OpenH264],
    )
    .unwrap();
    let quarter = rate.slowed(NonZeroU32::new(4).unwrap()).unwrap();
    for ((window, zoom), track) in windows.iter().zip(&zooms).zip(&tracks) {
        let decoded = Decoder::open(&zoom.path, Threads::Automatic).unwrap();
        assert_eq!(decoded.stream().frame_rate(), quarter);
        let similarity = compare_zoom(&comparison, window, track, &zoom.path);
        println!(
            "zoom {}: {:.4} on {} blocks",
            zoom.path.display(),
            similarity.mean(),
            similarity.blocks
        );
        assert!(similarity.mean() >= ZOOM_THRESHOLD, "{similarity:?}");
    }
}

#[test]
#[ignore = "slow"]
fn the_annotated_extract_has_every_frame_at_its_source_pts_and_keeps_the_source_outside_the_drawings() {
    let path = reference::fixture::fixture(&format!("{EXTRACT}.mp4")).unwrap();
    let read = sequential::read(&path);
    let tracks = Recording::read(&committed(EXTRACT)).unwrap().tracks;
    let source = Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap();
    let rate = read.timeline.rate();
    let config = RenderConfig::default();
    let comparison = Comparison {
        source: &source,
        read: &read,
        tracks: &tracks,
        trail_frames: (config.trail_s() * rate.get().to_f64()).round() as usize,
        pad: f64::from(config.box_pad()),
    };
    let source_duration = Decoder::open(&path, Threads::Automatic).unwrap().stream().duration();
    let whole = Window {
        track: 0,
        span: Span::new(FrameIndex::new(0), FrameIndex::new(read.timeline.frame_count() - 1)).unwrap(),
    };
    let decoder = Decoder::open(&path, Threads::Automatic).unwrap();
    let stream = decoder.stream();
    let available = Fallback::for_this_platform()
        .trial(stream.width(), stream.height(), stream.frame_rate().get())
        .unwrap();
    for codec in available.codecs() {
        let folder = tempfile::tempdir().unwrap();
        let annotated = folder.path().join("annotated.mp4");
        let outputs = Outputs {
            clips: Vec::new(),
            zooms: Vec::new(),
            annotated: Some(annotated.clone()),
        };
        render(&source, &tracks, &outputs, config, &[codec]).unwrap();
        let written = sequential::read(&annotated);
        assert_eq!(written.timeline.frame_count(), 1230);
        let pts = |read: &Sequential| -> Vec<Option<Pts>> {
            (0..read.timeline.frame_count())
                .map(|frame| read.timeline.pts(FrameIndex::new(frame)))
                .collect()
        };
        assert_eq!(pts(&written), pts(&read));
        let duration = Decoder::open(&annotated, Threads::Automatic)
            .unwrap()
            .stream()
            .duration();
        assert_eq!(duration, source_duration);
        let similarity = compare_clip(&comparison, &whole, &annotated);
        println!(
            "{codec:?} annotated: {:.4} on {} blocks",
            similarity.mean(),
            similarity.blocks
        );
        assert!(similarity.mean() >= threshold(codec), "{codec:?}: {similarity:?}");
    }
}
