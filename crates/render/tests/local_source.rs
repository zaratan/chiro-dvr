#[path = "support/sequential.rs"]
mod sequential;
#[path = "support/windows.rs"]
mod windows;

use std::io::{self, Write};
use std::path::Path;

use reference::location::{recorded_video, recording_directories};
use reference::recording::SUMMARY_FILE;
use reference::video::VideoSummary;
use render::source::{Origin, Source};
use types::frame::FrameIndex;
use types::span::Span;

const CLIP_MARGIN_SECONDS: f64 = 1.5;
const CROSSING_A_LOSS: (&str, usize, usize) = ("video_103.mp4", 6580, 6684);
const DAMAGED: [&str; 2] = ["video_123.mp4", "video_124.mp4"];

#[cfg(test)]
fn clip_windows(directory: &Path, summary: &VideoSummary) -> Vec<Span> {
    let tracks = reference::tracks::read(&directory.join(&summary.files.tracks)).unwrap();
    let rate = summary.stream.avg_frame_rate;
    let margin = (CLIP_MARGIN_SECONDS * f64::from(rate.num()) / f64::from(rate.den())).round() as usize;
    let last_frame = FrameIndex::new(summary.counts.frames - 1);
    tracks
        .iter()
        .map(|track| {
            let first = FrameIndex::new(track.first().frame.get().saturating_sub(margin));
            let last = FrameIndex::new(track.last().frame.get() + margin).min(last_frame);
            Span::new(first, last).unwrap()
        })
        .collect()
}

#[cfg(test)]
fn serve_all(name: &str, windows: &[Span]) {
    let path = recorded_video(name).unwrap();
    let read = sequential::read(&path);
    let source = Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap();
    let mut out = io::stdout().lock();
    for window in windows {
        let origin = windows::assert_served_like_a_sequential_reading(&source, &read, *window);
        assert!(
            matches!(origin, Origin::Key(key) if !read.errors.contains(&key)),
            "{name} {window:?}"
        );
        let last_key = read.keys.iter().rev().find(|key| **key <= window.first());
        let passed_over = last_key.filter(|key| read.errors.contains(key));
        writeln!(
            out,
            "{name} {window:?}: {origin:?}, key frame in error passed over: {passed_over:?}, {} frames equal",
            window.frame_count()
        )
        .unwrap();
    }
}

#[test]
#[ignore = "local"]
fn the_only_clip_across_lost_frames_serves_the_pixels_of_a_sequential_reading() {
    let (name, first, last) = CROSSING_A_LOSS;
    serve_all(
        name,
        &[Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()],
    );
}

#[test]
#[ignore = "local"]
fn clips_over_damaged_spans_serve_the_pixels_of_a_sequential_reading() {
    for directory in recording_directories().unwrap() {
        let summary = VideoSummary::read(&directory.join(SUMMARY_FILE)).unwrap();
        let name = summary.video.name.as_str();
        if !DAMAGED.contains(&name) {
            continue;
        }
        let damaged: Vec<Span> = clip_windows(&directory, &summary)
            .into_iter()
            .filter(|window| {
                summary
                    .damaged_spans
                    .iter()
                    .any(|span| span.intersection(*window).is_some())
            })
            .collect();
        assert!(!damaged.is_empty(), "{name}");
        serve_all(name, &damaged);
    }
}
