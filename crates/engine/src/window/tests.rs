use std::num::NonZeroUsize;

use proptest::prelude::*;

use super::{Prepared, Window, most_sampled};

fn window(frames: u64) -> Window {
    let mut window = Window::new(NonZeroUsize::new(15).unwrap(), NonZeroUsize::new(3).unwrap(), 4);
    for sum in 0..frames {
        let mut slot = window.slot();
        slot.sum = sum;
        window.push(slot);
    }
    window
}

fn sums<'a>(frames: impl Iterator<Item = &'a Prepared>) -> Vec<u64> {
    frames.map(|frame| frame.sum).collect()
}

#[test]
fn a_target_samples_one_frame_in_three_from_the_start_of_its_window() {
    let window = window(60);

    assert_eq!(
        sums(window.sampled(30, 59)),
        [15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45]
    );
}

#[test]
fn the_window_is_cut_at_both_ends_of_the_video() {
    let window = window(20);

    assert_eq!(sums(window.sampled(2, 19)), [0, 3, 6, 9, 12, 15]);
    assert_eq!(sums(window.sampled(18, 19)), [3, 6, 9, 12, 15, 18]);
}

#[test]
fn released_frames_are_recycled_without_allocating_again() {
    let mut window = window(40);
    let ninth = window.frame(9).units.as_ptr();

    window.release_before(10);
    let recycled = window.slot();

    assert_eq!(window.pushed(), 40);
    assert_eq!(window.frame(10).sum, 10);
    assert_eq!(recycled.units.as_ptr(), ninth);
    assert_eq!(window.spare.len(), 9);
}

#[test]
fn releasing_more_than_was_pushed_stops_at_the_last_frame() {
    let mut window = window(3);

    window.release_before(10);

    assert_eq!(window.pushed(), 3);
    assert_eq!(window.spare.len(), 3);
}

proptest! {
    #[test]
    fn no_target_samples_more_than_the_most_and_an_inner_one_samples_exactly_that(
        half in 1_usize..40,
        step in 1_usize..12,
        target in 0_usize..200,
    ) {
        let (half, step) = (NonZeroUsize::new(half).unwrap(), NonZeroUsize::new(step).unwrap());
        let mut window = Window::new(half, step, 1);
        for _ in 0..=200 + 2 * half.get() {
            let slot = window.slot();
            window.push(slot);
        }
        let most = most_sampled(half, step).get();
        let inner = target + half.get();

        prop_assert!(window.sampled(target, 199).count() <= most);
        prop_assert_eq!(window.sampled(inner, inner + half.get()).count(), most);
        prop_assert_eq!(most, (2 * half.get() + 1).div_ceil(step.get()));
    }
}
