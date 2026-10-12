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

fn lanes(group: &Group) -> Vec<(usize, usize)> {
    group
        .members()
        .iter()
        .map(|member| (member.window.track, member.lane))
        .collect()
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
fn a_window_takes_the_first_lane_whose_last_window_has_ended_as_v0_1_1_did() {
    let found = groups(&windows(&[(0, 10), (5, 20), (11, 30), (21, 25), (8, 9)]));
    assert_eq!(found.len(), 1);
    assert_eq!(lanes(&found[0]), [(0, 0), (1, 1), (4, 2), (2, 0), (3, 1)]);
    assert_eq!(found[0].lanes(), 3);
}

#[test]
fn lanes_start_again_from_the_first_in_each_group() {
    let found = groups(&windows(&[(0, 10), (5, 20), (40, 50)]));
    assert_eq!(lanes(&found[1]), [(2, 0)]);
    assert_eq!(found[1].lanes(), 1);
}

#[test]
fn a_group_holds_at_most_eight_hardware_lanes() {
    let crowded: Vec<(usize, usize)> = (0..10).map(|start| (start, 100)).collect();
    let found = groups(&windows(&crowded));
    assert_eq!((found[0].lanes(), found[0].hardware_lanes()), (10, 8));
    assert_eq!(groups(&windows(&[(0, 10), (5, 20)]))[0].hardware_lanes(), 2);
}

#[test]
fn the_hardware_sessions_a_group_still_needs_only_fall_as_its_windows_close() {
    let found = groups(&windows(&[(0, 10), (5, 20), (11, 30), (21, 25), (8, 9)]));
    let needed: Vec<usize> = [0, 9, 10, 21, 26, 30, 31]
        .iter()
        .map(|frame| found[0].hardware_lanes_from(FrameIndex::new(*frame)))
        .collect();
    assert_eq!(needed, [3, 3, 2, 2, 1, 1, 0]);
    assert_eq!(found[0].hardware_lanes(), 3);
}

#[test]
fn lanes_beyond_the_eighth_need_no_hardware_session_at_any_frame() {
    let crowded: Vec<(usize, usize)> = (0..10).map(|start| (start, 100 - start)).collect();
    let found = groups(&windows(&crowded));
    let needed: Vec<usize> = (0..=101)
        .map(|frame| found[0].hardware_lanes_from(FrameIndex::new(frame)))
        .collect();
    assert_eq!(needed[0], 8);
    assert!(needed.windows(2).all(|pair| pair[0] >= pair[1]), "{needed:?}");
    assert_eq!((needed[93], needed[94], needed[100], needed[101]), (8, 7, 1, 0));
}

#[test]
fn windows_starting_together_keep_the_order_of_their_tracks() {
    let found = groups(&windows(&[(5, 10), (5, 10), (0, 10)]));
    assert_eq!(lanes(&found[0]), [(2, 0), (0, 1), (1, 2)]);
}

fn reference_windows(name: &str) -> (Vec<usize>, usize) {
    let directory = reference::location::committed(name);
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

#[test]
fn a_window_starting_on_the_last_frame_of_another_cannot_share_its_lane() {
    let found = groups(&windows(&[(0, 10), (10, 20), (11, 30)]));
    assert_eq!(lanes(&found[0]), [(0, 0), (1, 1), (2, 0)]);
}
