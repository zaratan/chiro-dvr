use std::collections::BTreeSet;
use std::num::NonZeroUsize;
use std::path::Path;

use media::decode::{Packet, Threads};
use proptest::prelude::*;
use types::frame::FrameIndex;
use types::span::Span;

use super::{Damage, edges, spans};
use crate::input::{Error, Input, Reading};
use crate::lost::Losses;
use crate::testing::{self, FRAMES};

#[path = "../../tests/support/slices.rs"]
mod slices;

const KEYS: [usize; 3] = [0, 15, 30];

fn spanned(starts: &[usize], keys: &[usize], frame_count: usize, margin: usize) -> Vec<Span> {
    spans(starts, keys, frame_count, margin).unwrap()
}

fn damage(reading: &Reading, margin: usize) -> Damage {
    Damage::of(reading, margin).unwrap()
}

fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

fn frames(indices: &[usize]) -> Vec<FrameIndex> {
    indices.iter().copied().map(FrameIndex::new).collect()
}

fn read(input: Input) -> Reading {
    input.read(|_, _| Ok::<(), Error>(())).unwrap()
}

fn open(path: &Path, threads: Threads) -> Input {
    Input::open_any_size(path, threads).unwrap()
}

#[test]
fn damage_lasts_until_the_frame_before_the_next_key_frame() {
    assert_eq!(spanned(&[17], &KEYS, 45, 0), [span(17, 29)]);
}

#[test]
fn damage_in_the_last_group_lasts_until_the_end() {
    assert_eq!(spanned(&[33], &KEYS, 45, 0), [span(33, 44)]);
}

#[test]
fn two_damaged_frames_in_one_group_make_one_span() {
    assert_eq!(spanned(&[17, 22], &KEYS, 45, 0), [span(17, 29)]);
}

#[test]
fn damage_on_the_key_frame_after_a_span_extends_it() {
    assert_eq!(spanned(&[17, 30], &KEYS, 45, 0), [span(17, 44)]);
}

#[test]
fn damage_on_the_last_frame_gives_a_one_frame_span() {
    assert_eq!(spanned(&[44], &KEYS, 45, 0), [span(44, 44)]);
}

#[test]
fn a_video_without_frames_has_no_damage_whatever_it_lost() {
    let losses = Losses {
        before_first: 3,
        at_jumps: 0,
        after_last: 2,
    };
    let starts: Vec<usize> = edges(losses, 0).collect();
    assert_eq!(spanned(&starts, &[], 0, 5), []);
}

#[test]
fn the_margin_widens_both_sides_before_spans_merge() {
    let keys = [0, 15, 30, 45];
    assert_eq!(spanned(&[17, 35], &keys, 60, 3), [span(14, 47)]);
    assert_eq!(spanned(&[17, 37], &keys, 60, 3), [span(14, 32), span(34, 47)]);
}

#[test]
fn the_margin_stays_inside_the_video() {
    assert_eq!(spanned(&[2, 40], &KEYS, 45, 15), [span(0, 44)]);
    assert_eq!(spanned(&[2], &[0, 15, 30, 45, 60, 75], 90, 15), [span(0, 29)]);
}

#[test]
fn a_loss_before_the_first_frame_starts_damage_on_it_like_a_jump() {
    let before = Losses {
        before_first: 10,
        ..Losses::default()
    };
    assert_eq!(edges(before, 23).collect::<Vec<_>>(), [0]);
    assert_eq!(edges(Losses::default(), 23).count(), 0);
}

#[test]
fn a_loss_after_the_last_frame_damages_only_the_margin_before_the_end() {
    let after = Losses {
        after_last: 7,
        ..Losses::default()
    };
    let starts: Vec<usize> = edges(after, 45).collect();
    assert_eq!(starts, [45]);
    assert_eq!(spanned(&starts, &KEYS, 45, 3), [span(42, 44)]);
    assert_eq!(spanned(&starts, &KEYS, 45, 0), []);
}

proptest! {
    #[test]
    fn spans_never_touch_and_stay_inside_the_video(
        starts in proptest::collection::btree_set(0_usize..45, 0..45),
        margin in 0_usize..20,
    ) {
        let starts: Vec<usize> = starts.into_iter().collect();
        let found = spanned(&starts, &KEYS, 45, margin);
        prop_assert!(found.iter().zip(found.iter().skip(1)).all(|(a, b)| a.last().get() + 1 < b.first().get()));
        prop_assert!(found.iter().all(|span| span.last().get() <= 44));
        prop_assert!(starts.iter().all(|start| found.iter().any(|span| span.contains(FrameIndex::new(*start)))));
    }
}

#[test]
fn a_clean_video_has_no_damaged_frame() {
    let folder = tempfile::tempdir().unwrap();
    let reading = read(open(&testing::encode(folder.path(), FRAMES), Threads::Automatic));
    let found = damage(&reading, 4);
    assert_eq!(
        found,
        Damage {
            error_frames: Vec::new(),
            jump_frames: Vec::new(),
            spans: Vec::new(),
        }
    );
}

#[test]
fn exactly_the_frames_whose_slice_was_cut_are_in_error_with_one_thread_or_many() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), FRAMES);
    for threads in [Threads::Automatic, Threads::Exactly(NonZeroUsize::MIN)] {
        let cut = BTreeSet::from([0, 10, 20, 21, 29]);
        let reading = read(open(&path, threads).with_seam(slices::cutting_the_last_third_of(cut)));
        let found = damage(&reading, 0);
        assert_eq!(found.error_frames(), frames(&[0, 10, 20, 21, 29]), "{threads:?}");
        assert_eq!(found.jump_frames(), [], "{threads:?}");
        assert_eq!(found.spans(), [span(0, 29)], "{threads:?}");
    }
}

#[test]
fn the_frame_whose_timestamp_jumps_starts_damage() {
    let folder = tempfile::tempdir().unwrap();
    let reading = read(open(&testing::damaged(folder.path(), 2, 3), Threads::Automatic));
    let ticks: Vec<i64> = reading.frames().iter().map(|info| info.pts().unwrap().get()).collect();
    let step = ticks[1] - ticks[0];
    let jumping = ticks.iter().zip(&ticks[1..]).position(|(a, b)| b - a > step).unwrap() + 1;
    let found = damage(&reading, 2);
    assert_eq!(found.jump_frames(), frames(&[jumping]));
    let first = found
        .error_frames()
        .iter()
        .chain(found.jump_frames())
        .min()
        .unwrap()
        .get();
    assert_eq!(found.spans(), [span(first - 2, reading.frames().len() - 1)]);
}

#[test]
fn frames_unreadable_after_the_last_one_damage_the_margin_before_the_end() {
    let folder = tempfile::tempdir().unwrap();
    let path = testing::encode(folder.path(), FRAMES);
    let unreadable_end = |packet: &mut Packet<'_>| {
        if packet.ordinal() >= 27 {
            packet.data_mut().unwrap().fill(0xff);
        }
    };
    let reading = read(open(&path, Threads::Automatic).with_seam(Box::new(unreadable_end)));
    let found = damage(&reading, 3);
    assert_eq!(reading.frames().len(), 27);
    assert_eq!(reading.losses().after_last, 3);
    assert_eq!((found.error_frames(), found.jump_frames()), (&[][..], &[][..]));
    assert_eq!(found.spans(), [span(24, 26)]);
}

#[test]
fn frames_lost_before_the_first_one_damage_the_start_as_far_as_the_end_of_its_group() {
    let path = reference::fixture::fixture("two_gops.mp4").unwrap();
    let cut_first_key = |packet: &mut Packet<'_>| {
        if packet.ordinal() == 0 {
            let half = packet.size() / 2;
            packet.shrink(half);
        }
    };
    let reading = read(open(&path, Threads::Automatic).with_seam(Box::new(cut_first_key)));
    let found = damage(&reading, 0);
    assert_eq!(reading.frames().len(), 10);
    assert_eq!((found.error_frames(), found.jump_frames()), (&[][..], &[][..]));
    assert_eq!(found.spans(), [span(0, 9)]);
}

#[test]
fn a_frame_both_in_error_and_after_a_jump_is_reported_once() {
    let damage = Damage {
        error_frames: frames(&[17, 40]),
        jump_frames: frames(&[40, 55]),
        spans: Vec::new(),
    };
    assert_eq!(damage.reported_frames(), 3);
}
