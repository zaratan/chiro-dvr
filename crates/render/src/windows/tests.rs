use reference::recording::SUMMARY_FILE;
use reference::video::VideoSummary;
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::span::Span;

use super::{Group, Window, clip_windows, groups};
use crate::config::RenderConfig;
use crate::testing::{thirty, track};

fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

fn windows(spans: &[(usize, usize)]) -> Vec<Window> {
    spans
        .iter()
        .enumerate()
        .map(|(track, &(first, last))| Window {
            track,
            span: span(first, last),
        })
        .collect()
}

fn tracks_of(group: &Group) -> Vec<usize> {
    group.members().iter().map(|window| window.track).collect()
}

#[test]
fn a_window_reaches_45_frames_around_its_track_and_stops_at_both_ends_of_the_video() {
    let tracks = [
        track(1, &[(100, 0.0, 0.0), (120, 0.0, 0.0)]),
        track(2, &[(10, 0.0, 0.0), (30, 0.0, 0.0)]),
        track(3, &[(950, 0.0, 0.0), (990, 0.0, 0.0)]),
    ];
    let found = clip_windows(&tracks, 1000, RenderConfig::default(), thirty());
    assert_eq!(found, windows(&[(55, 165), (0, 75), (905, 999)]));
}

#[test]
fn a_track_beyond_the_video_has_no_window_and_an_empty_video_has_none() {
    let tracks = [track(1, &[(5_000, 0.0, 0.0)])];
    assert_eq!(clip_windows(&tracks, 1000, RenderConfig::default(), thirty()), []);
    assert_eq!(clip_windows(&tracks, 0, RenderConfig::default(), thirty()), []);
}

#[test]
fn windows_that_overlap_or_touch_share_a_group_and_a_gap_starts_another() {
    let found = groups(&windows(&[(0, 10), (11, 20), (15, 30), (32, 40)]));
    let spans: Vec<Span> = found.iter().map(Group::span).collect();
    assert_eq!(spans, [span(0, 30), span(32, 40)]);
}

#[test]
fn windows_starting_together_keep_the_order_of_their_tracks() {
    let found = groups(&windows(&[(5, 10), (5, 10), (0, 10)]));
    assert_eq!(tracks_of(&found[0]), [2, 0, 1]);
}

fn reference_windows(name: &str) -> (Vec<usize>, usize) {
    let directory = reference::location::committed(reference::location::Reference::V011, name);
    let summary = VideoSummary::read(&directory.join(SUMMARY_FILE)).unwrap();
    let tracks = reference::tracks::read(&directory.join(&summary.files.tracks)).unwrap();
    let rate = FrameRate::new(summary.stream.avg_frame_rate).unwrap();
    let found = clip_windows(&tracks, summary.counts.frames, RenderConfig::default(), rate);
    let lengths = found.iter().map(|window| window.span.frame_count()).collect();
    (lengths, groups(&found).len())
}

#[test]
fn the_reference_tracks_of_the_extract_and_the_092_give_the_lengths_of_the_v0_1_1_clips() {
    assert_eq!(
        reference_windows("video_092_original_3m24-4m05"),
        (vec![380, 108, 115, 117], 3)
    );
    assert_eq!(
        reference_windows("video_092_original"),
        (
            vec![113, 131, 99, 125, 129, 123, 111, 106, 105, 134, 380, 108, 115, 117],
            12
        )
    );
}
