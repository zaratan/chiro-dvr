use std::num::NonZeroUsize;

use media::frame::{ColorRange, YuvFrame};
use types::frame::FrameIndex;

use super::{SUMMARY_FRAMES, SummaryFrames, ranks};

fn announced(frames: usize) -> NonZeroUsize {
    NonZeroUsize::new(frames).unwrap()
}

fn offered(announced_frames: usize, decoded: usize) -> Vec<usize> {
    let mut summary = SummaryFrames::new(announced(announced_frames));
    let mut frame = YuvFrame::new(4, 4, ColorRange::Limited).unwrap();
    for index in 0..decoded {
        frame.y_mut().fill(index as u8);
        summary.offer(FrameIndex::new(index), &frame);
    }
    summary
        .into_frames()
        .iter()
        .map(|(index, frame)| {
            assert_eq!(usize::from(frame.y()[0]), index.get() % 256);
            index.get()
        })
        .collect()
}

#[test]
fn twenty_five_frames_are_taken_at_the_middle_of_twenty_five_equal_parts() {
    let taken = ranks(announced(9011));

    assert_eq!(taken.len(), SUMMARY_FRAMES);
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
fn frames_lost_at_the_end_leave_fewer_summary_frames() {
    assert_eq!(offered(50, 10), [1, 3, 5, 7, 9]);
}

#[test]
fn an_absurd_announced_count_gives_no_rank_rather_than_overflowing() {
    assert!(ranks(announced(usize::MAX)).len() < SUMMARY_FRAMES);
}
