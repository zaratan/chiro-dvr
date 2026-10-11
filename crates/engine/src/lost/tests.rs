use proptest::prelude::*;
use types::frame::Pts;

use super::{Losses, losses};

fn lost_frames(pts: &[Option<Pts>]) -> usize {
    losses(pts, None, 0).at_jumps
}

fn known(ticks: impl IntoIterator<Item = i64>) -> Vec<Option<Pts>> {
    ticks.into_iter().map(|tick| Some(Pts::new(tick))).collect()
}

#[test]
fn a_regular_sequence_loses_nothing() {
    assert_eq!(lost_frames(&known((0..100).map(|frame| frame * 333))), 0);
}

#[test]
fn a_jump_of_fourteen_steps_in_jittered_timestamps_loses_thirteen_frames_like_the_122() {
    let mut ticks: Vec<i64> = (0..60).map(|frame| frame * 333 + frame / 3).collect();
    let jump = 4662 + ticks[29] - ticks[30];
    for tick in &mut ticks[30..] {
        *tick += jump;
    }
    assert_eq!(lost_frames(&known(ticks)), 13);
}

#[test]
fn a_long_jump_with_a_fractional_step_is_counted_exactly() {
    let frames = (0..30).chain(1530..1560);
    let ticks = frames.map(|frame| frame * 333 + frame / 3);
    assert_eq!(lost_frames(&known(ticks)), 1500);
}

#[test]
fn an_unknown_timestamp_between_two_frames_is_not_a_jump() {
    let pts = [
        Some(Pts::new(0)),
        Some(Pts::new(1000)),
        None,
        Some(Pts::new(3000)),
        Some(Pts::new(4000)),
    ];
    assert_eq!(lost_frames(&pts), 0);
}

#[test]
fn repeated_timestamps_are_not_jumps() {
    assert_eq!(lost_frames(&known([0, 0, 1000, 1000, 2000, 2000, 3000])), 0);
}

#[test]
fn fewer_than_two_known_timestamps_lose_nothing() {
    assert_eq!(lost_frames(&[]), 0);
    assert_eq!(lost_frames(&[None, Some(Pts::new(5)), None]), 0);
}

#[test]
fn timestamps_going_backwards_lose_nothing() {
    assert_eq!(lost_frames(&known([4000, 3000, 2000, 1000])), 0);
}

proptest! {
    #[test]
    fn frames_lost_in_a_minority_of_intervals_are_all_counted(
        step in 1_i64..10_000,
        start in -1_000_000_i64..1_000_000,
        decoded in 30_usize..300,
        jumps in proptest::collection::vec((any::<usize>(), 1_i64..20), 0..10),
    ) {
        let mut lost_after = vec![0_i64; decoded];
        for (position, lost) in &jumps {
            lost_after[position % (decoded - 1)] += lost;
        }
        let ticks = lost_after.iter().scan(-1_i64, |frame, lost| {
            *frame += 1;
            let tick = start + step * *frame;
            *frame += lost;
            Some(tick)
        });
        let lost: i64 = jumps.iter().map(|(_, lost)| lost).sum();
        prop_assert_eq!(lost_frames(&known(ticks)), usize::try_from(lost).unwrap());
    }
}

#[test]
fn frames_missing_before_the_first_timestamp_are_counted_from_the_start_and_the_step() {
    let pts = known((10..20).map(|frame| frame * 512));
    let found = losses(&pts, Some(Pts::new(0)), 3);
    assert_eq!(
        found,
        Losses {
            before_first: 10,
            at_jumps: 0,
            after_last: 3,
        }
    );
    assert_eq!(found.total(), 13);
}

#[test]
fn unknown_timestamps_at_the_head_are_frames_already_decoded_not_losses() {
    let mut pts = vec![None, None];
    pts.extend(known((2..12).map(|frame| frame * 512)));
    assert_eq!(losses(&pts, Some(Pts::new(0)), 0).before_first, 0);
}

#[test]
fn an_unknown_start_or_a_first_frame_at_the_start_loses_nothing_before_it() {
    let pts = known((5..15).map(|frame| frame * 512));
    assert_eq!(losses(&pts, None, 0).before_first, 0);
    assert_eq!(losses(&pts, Some(Pts::new(5 * 512)), 0).before_first, 0);
    assert_eq!(losses(&pts, Some(Pts::new(9 * 512)), 0).before_first, 0);
}

#[test]
fn without_a_step_only_the_unreadable_packets_after_the_last_frame_count() {
    assert_eq!(
        losses(&[Some(Pts::new(5120))], Some(Pts::new(0)), 4),
        Losses {
            before_first: 0,
            at_jumps: 0,
            after_last: 4,
        }
    );
}
