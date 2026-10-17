use std::path::{Path, PathBuf};

use draw::color::{PALETTE, Rgb};
use tempfile::TempDir;
use types::track::Track;

use super::{Drawn, drawn};
use crate::clips::{Clip, Outputs};
use crate::colored::Colored;
use crate::config::RenderConfig;
use crate::sequential;
use crate::source::Source;
use crate::testing::{self, FRAMES, track};
use crate::windows::clip_windows;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;

fn tenth_of_a_second() -> RenderConfig {
    RenderConfig::new(12, 1.0, 0.1, 300).unwrap()
}

fn source(folder: &Path) -> Source {
    let path = testing::binoculars_of(folder, WIDTH, HEIGHT);
    let read = sequential::read(&path);
    Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap()
}

fn painted(source: &Source, tracks: &[Track], colors: &[Option<Rgb>], outputs: &Outputs) -> Vec<Drawn> {
    drawn(
        source,
        Colored::new(tracks, colors).unwrap(),
        outputs,
        tenth_of_a_second(),
    )
    .unwrap()
}

fn uncolored(source: &Source, tracks: &[Track], outputs: &Outputs) -> Vec<Drawn> {
    painted(source, tracks, &[], outputs)
}

fn crossing_from(first: usize, row: f64) -> Track {
    let points: Vec<(usize, f64, f64)> = (first..=first + 10)
        .map(|frame| (frame, 20.0 + 4.0 * frame as f64, row))
        .collect();
    track(1, &points)
}

fn crossing(row: f64) -> Track {
    crossing_from(10, row)
}

fn clip(folder: &Path, name: &str) -> Clip {
    Clip {
        track: 0,
        path: folder.join(name),
    }
}

fn clip_only(folder: &Path) -> Outputs {
    Outputs {
        clips: vec![clip(folder, "clip.mp4")],
        zooms: Vec::new(),
        annotated: None,
    }
}

#[test]
fn every_clip_and_zoom_gets_one_fingerprint_per_frame_of_its_window_sorted_by_path() {
    let folder = TempDir::new().unwrap();
    let source = source(folder.path());
    let tracks = [crossing(60.0)];
    let outputs = Outputs {
        clips: vec![clip(folder.path(), "b.mp4")],
        zooms: vec![clip(folder.path(), "a_zoom.mp4")],
        annotated: None,
    };

    let found = uncolored(&source, &tracks, &outputs);

    let window = clip_windows(&tracks, source.frame_count(), tenth_of_a_second(), source.rate())[0];
    let paths: Vec<PathBuf> = found.iter().map(|drawn| drawn.path.clone()).collect();
    assert_eq!(paths, [folder.path().join("a_zoom.mp4"), folder.path().join("b.mp4")]);
    for drawn in &found {
        assert_eq!(drawn.fingerprints.len(), window.span.frame_count(), "{drawn:?}");
    }
    assert_ne!(found[0].fingerprints, found[1].fingerprints);
}

#[test]
fn moving_the_track_changes_the_drawn_frames_exactly_over_its_points() {
    let folder = TempDir::new().unwrap();
    let source = source(folder.path());
    let outputs = clip_only(folder.path());

    let high = uncolored(&source, &[crossing(60.0)], &outputs);
    let again = uncolored(&source, &[crossing(60.0)], &outputs);
    let low = uncolored(&source, &[crossing(160.0)], &outputs);

    let window = clip_windows(
        &[crossing(60.0)],
        source.frame_count(),
        tenth_of_a_second(),
        source.rate(),
    )[0];
    let first = window.span.first().get();
    assert_eq!(high, again);
    for (offset, (high, low)) in high[0].fingerprints.iter().zip(&low[0].fingerprints).enumerate() {
        let frame = first + offset;
        assert_eq!(high != low, (10..=20).contains(&frame), "frame {frame}");
    }
}

#[test]
fn the_colour_of_a_track_changes_its_clip_its_zoom_and_the_annotated_video() {
    let folder = TempDir::new().unwrap();
    let source = source(folder.path());
    let outputs = Outputs {
        clips: vec![clip(folder.path(), "clip.mp4")],
        zooms: vec![clip(folder.path(), "zoom.mp4")],
        annotated: Some(folder.path().join("annotated.mp4")),
    };
    let tracks = [crossing(60.0)];

    let red = painted(&source, &tracks, &[None], &outputs);
    let cyan = painted(&source, &tracks, &[Some(PALETTE[1])], &outputs);

    assert_eq!((red.len(), cyan.len()), (3, 3));
    for (red, cyan) in red.iter().zip(&cyan) {
        assert_ne!(red.fingerprints, cyan.fingerprints, "{}", red.path.display());
    }
}

#[test]
fn the_zoom_of_a_track_takes_the_colour_of_that_track_whatever_the_order_of_the_windows() {
    let folder = TempDir::new().unwrap();
    let source = source(folder.path());
    let second: Vec<(usize, f64, f64)> = (20..=30)
        .map(|frame| (frame, 20.0 + 4.0 * frame as f64, 160.0))
        .collect();
    let tracks = [crossing(60.0), track(2, &second)];
    let outputs = Outputs {
        clips: Vec::new(),
        zooms: vec![Clip {
            track: 1,
            path: folder.path().join("zoom.mp4"),
        }],
        annotated: None,
    };
    let zoom = |colors: &[Option<Rgb>]| painted(&source, &tracks, colors, &outputs)[0].fingerprints.clone();
    let (yellow, cyan, orange) = (Some(PALETTE[0]), Some(PALETTE[1]), Some(PALETTE[2]));

    assert_eq!(zoom(&[yellow, cyan]), zoom(&[orange, cyan]));
    assert_ne!(zoom(&[yellow, cyan]), zoom(&[yellow, orange]));
}

#[test]
fn the_annotated_video_gets_one_fingerprint_per_frame_of_the_source() {
    let folder = TempDir::new().unwrap();
    let source = source(folder.path());
    let annotated = folder.path().join("annotated.mp4");
    let outputs = Outputs {
        clips: Vec::new(),
        zooms: Vec::new(),
        annotated: Some(annotated.clone()),
    };

    let found = uncolored(&source, &[crossing(60.0)], &outputs);

    assert_eq!(found.len(), 1);
    assert_eq!(found[0].path, annotated);
    assert_eq!(found[0].fingerprints.len(), FRAMES);
}

#[test]
fn two_clips_of_one_group_each_get_the_shared_drawn_frames_from_their_own_first_frame() {
    let folder = TempDir::new().unwrap();
    let source = source(folder.path());
    let tracks = [crossing_from(10, 60.0), crossing_from(15, 160.0).with_id(2)];
    let outputs = Outputs {
        clips: vec![
            clip(folder.path(), "a.mp4"),
            Clip {
                track: 1,
                path: folder.path().join("b.mp4"),
            },
        ],
        zooms: Vec::new(),
        annotated: None,
    };

    let found = uncolored(&source, &tracks, &outputs);

    let windows = clip_windows(&tracks, source.frame_count(), tenth_of_a_second(), source.rate());
    let (first, second) = (windows[0].span, windows[1].span);
    assert!(second.first() <= first.last());
    assert_eq!(found[0].fingerprints.len(), first.frame_count());
    assert_eq!(found[1].fingerprints.len(), second.frame_count());
    let offset = second.first().get() - first.first().get();
    let shared = first.last().get() - second.first().get() + 1;
    assert_eq!(
        found[1].fingerprints[..shared],
        found[0].fingerprints[offset..offset + shared]
    );
}
