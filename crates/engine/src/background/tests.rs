use std::num::NonZeroUsize;

use media::frame::{ColorRange, YuvFrame};
use proptest::prelude::*;
use types::frame::FrameIndex;
use types::span::Span;

use super::{BACKGROUND_FRAMES, BackgroundFrames, median, middle, ranks};

fn announced(frames: usize) -> NonZeroUsize {
    NonZeroUsize::new(frames).unwrap()
}

fn flat(y: u8, u: u8, v: u8) -> YuvFrame {
    let mut frame = YuvFrame::new(8, 8, ColorRange::Limited).unwrap();
    frame.y_mut().fill(y);
    frame.u_mut().fill(u);
    frame.v_mut().fill(v);
    frame
}

fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

fn offered(announced_frames: usize, decoded: usize) -> Vec<usize> {
    let mut frames = BackgroundFrames::new(announced(announced_frames));
    let mut frame = YuvFrame::new(4, 4, ColorRange::Limited).unwrap();
    for index in 0..decoded {
        frame.y_mut().fill(index as u8);
        frames.offer(FrameIndex::new(index), &frame);
    }
    frames
        .kept
        .iter()
        .map(|(index, frame)| {
            assert_eq!(usize::from(frame.y()[0]), index.get() % 256);
            index.get()
        })
        .collect()
}

fn background_of(levels: &[u8], damaged: &[Span]) -> Option<YuvFrame> {
    let mut frames = BackgroundFrames::new(announced(levels.len()));
    for (index, level) in levels.iter().enumerate() {
        frames.offer(FrameIndex::new(index), &flat(*level, 128, 128));
    }
    frames.into_background(damaged)
}

#[test]
fn twenty_five_frames_are_taken_at_the_middle_of_twenty_five_equal_parts() {
    let taken = ranks(announced(9011));

    assert_eq!(taken.len(), BACKGROUND_FRAMES);
    assert_eq!((taken[0], taken[1], taken[24]), (180, 540, 8830));
}

#[test]
fn a_video_shorter_than_twenty_five_frames_gives_every_frame() {
    assert_eq!(ranks(announced(10)), (0..10).collect::<Vec<_>>());
    assert_eq!(ranks(announced(1)), [0]);
}

#[test]
fn the_frames_at_those_ranks_are_kept_in_full_as_they_are_decoded() {
    assert_eq!(offered(50, 50), (0..25).map(|index| 2 * index + 1).collect::<Vec<_>>());
}

#[test]
fn frames_lost_at_the_end_leave_fewer_background_frames() {
    assert_eq!(offered(50, 10), [1, 3, 5, 7, 9]);
}

#[test]
fn an_absurd_announced_count_gives_no_rank_rather_than_overflowing() {
    assert!(ranks(announced(usize::MAX)).len() < BACKGROUND_FRAMES);
}

#[test]
fn the_median_background_removes_a_moving_object() {
    let frames: Vec<YuvFrame> = (0..25)
        .map(|index| {
            let mut frame = flat(100, 128, 128);
            frame.y_mut()[index % 64] = 250;
            frame
        })
        .collect();

    let background = median(&frames.iter().collect::<Vec<_>>()).unwrap();

    assert!(background.y().iter().all(|level| *level == 100));
}

#[test]
fn an_even_count_takes_the_half_sum_rounded_to_the_nearest_level() {
    assert_eq!(middle(&mut [10, 13]), Some(12));
    assert_eq!(middle(&mut [11, 10]), Some(11));
    assert_eq!(middle(&mut [9, 1, 4, 3]), Some(4));
    assert_eq!(middle(&mut [255, 255]), Some(255));
}

#[test]
fn each_chroma_plane_takes_its_own_median() {
    let frames = [flat(10, 20, 200), flat(11, 30, 190), flat(12, 40, 180)];

    let background = median(&frames.iter().collect::<Vec<_>>()).unwrap();

    assert_eq!((background.y()[0], background.u()[0], background.v()[0]), (11, 30, 190));
    assert_eq!(background.range(), ColorRange::Limited);
}

#[test]
fn no_kept_frame_gives_no_background() {
    assert_eq!(BackgroundFrames::new(announced(50)).into_background(&[]), None);
    assert_eq!(median(&[]), None);
    assert_eq!(middle(&mut []), None);
}

#[test]
fn frames_in_a_damaged_span_are_left_out_of_the_background() {
    let background = background_of(&[0, 0, 0, 90, 80], &[span(0, 2)]).unwrap();

    assert_eq!(background.y()[0], 85);
}

#[test]
fn every_kept_frame_counts_when_all_of_them_are_damaged() {
    let background = background_of(&[0, 50, 90], &[span(0, 1), span(2, 2)]).unwrap();

    assert_eq!(background.y()[0], 50);
}

proptest! {
    #[test]
    fn the_median_matches_the_middle_of_a_full_sort(mut values in proptest::collection::vec(any::<u8>(), 1..40)) {
        let mut sorted = values.clone();
        sorted.sort_unstable();
        let half = sorted.len() / 2;
        let expected = if sorted.len() % 2 == 1 {
            sorted[half]
        } else {
            ((u16::from(sorted[half - 1]) + u16::from(sorted[half])).div_ceil(2)) as u8
        };

        prop_assert_eq!(middle(&mut values), Some(expected));
    }
}
