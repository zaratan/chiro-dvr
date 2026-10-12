use std::num::NonZeroU32;

use types::rate::FrameRate;
use types::rational::Rational;
use types::span::Span;

use super::{StabilityConfig, crowd_limit, unstable_spans};
use crate::error::Error;
use crate::fake::{crowd, span};
use crate::setting::Setting;

fn rate(num: i32, den: i32) -> FrameRate {
    FrameRate::new(Rational::new(num, den).unwrap()).unwrap()
}

fn config(pad_s: f64) -> StabilityConfig {
    StabilityConfig::new(NonZeroU32::new(20), pad_s).unwrap()
}

fn spans_hiding(counts: &[usize], damaged: &[Span]) -> Vec<Span> {
    unstable_spans(&crowd(counts), damaged, rate(10, 1), config(0.2))
}

fn spans(counts: &[usize]) -> Vec<Span> {
    spans_hiding(counts, &[])
}

fn quiet_with(crowded: &[usize], frames: usize) -> Vec<usize> {
    (0..frames)
        .map(|frame| if crowded.contains(&frame) { 30 } else { 0 })
        .collect()
}

#[test]
fn frames_below_the_limit_are_kept() {
    let mut counts = vec![1; 41];
    counts[20] = 25;
    assert_eq!(spans(&counts), []);
}

#[test]
fn a_crowded_frame_is_ignored_with_its_padding() {
    let mut counts = vec![0; 41];
    counts[20] = 20;
    assert_eq!(spans(&counts), [span(18, 22)]);
}

#[test]
fn padding_is_rounded_to_frames_at_a_fractional_rate() {
    let mut counts = vec![0; 201];
    counts[100] = 30;
    let found = unstable_spans(&crowd(&counts), &[], rate(30_000, 1001), StabilityConfig::default());
    assert_eq!(found, [span(70, 130)]);
}

#[test]
fn the_limit_rises_with_the_usual_number_of_blobs() {
    let mut noisy = vec![10; 81];
    noisy[40] = 70;
    assert_eq!(spans(&noisy), []);
    noisy[40] = 80;
    assert_eq!(spans(&noisy), [span(38, 42)]);
}

#[test]
fn close_crowded_frames_form_one_span() {
    assert_eq!(spans(&quiet_with(&[10, 15], 40)), [span(8, 17)]);
}

#[test]
fn spans_separated_by_exactly_the_padding_are_merged() {
    assert_eq!(spans(&quiet_with(&[10, 17], 40)), [span(8, 19)]);
}

#[test]
fn spans_separated_by_one_frame_more_than_the_padding_stay_apart() {
    assert_eq!(spans(&quiet_with(&[10, 18], 40)), [span(8, 12), span(16, 20)]);
}

#[test]
fn spans_stop_at_the_first_and_last_frames() {
    assert_eq!(spans(&quiet_with(&[0, 21], 22)), [span(0, 2), span(19, 21)]);
}

#[test]
fn spans_stop_at_the_first_and_last_readable_frames_not_at_the_ends_of_the_video() {
    let found = spans_hiding(&quiet_with(&[10, 29], 40), &[span(0, 9), span(30, 39)]);
    assert_eq!(found, [span(10, 12), span(27, 29)]);
}

#[test]
fn a_padding_longer_than_the_video_covers_every_readable_frame() {
    let found = unstable_spans(&crowd(&[0, 30, 0]), &[], rate(10, 1), config(1e300));
    assert_eq!(found, [span(0, 2)]);
}

#[test]
fn no_max_blobs_disables_the_filter() {
    let mut counts = vec![0; 41];
    counts[20] = 500;
    let off = StabilityConfig::new(None, 1.0).unwrap();
    assert_eq!(unstable_spans(&crowd(&counts), &[], rate(10, 1), off), []);
}

#[test]
fn a_padding_that_is_negative_or_not_finite_is_refused() {
    for pad_s in [-0.1, f64::NAN, f64::INFINITY] {
        let error = StabilityConfig::new(NonZeroU32::new(20), pad_s).unwrap_err();
        assert!(matches!(
            error,
            Error::Setting {
                setting: Setting::PadS,
                ..
            }
        ));
        assert!(error.to_string().starts_with("pad_s is out of bounds"));
    }
}

#[test]
fn the_default_settings_are_those_of_the_reference() {
    let config = StabilityConfig::default();
    assert_eq!((config.max_blobs(), config.pad_s()), (NonZeroU32::new(20), 1.0));
}

#[test]
fn empty_frames_count_in_the_usual_number_of_blobs() {
    let twenty = NonZeroU32::new(20).unwrap();
    assert_eq!(crowd_limit(&[0, 0, 0, 10], twenty), 20);
    assert_eq!(crowd_limit(&[0, 0, 10, 10], twenty), 50);
    assert_eq!(crowd_limit(&[10, 0, 7], twenty), 62);
    assert_eq!(crowd_limit(&[], twenty), 20);
}

#[test]
fn a_video_without_readable_frames_has_no_unstable_span() {
    assert_eq!(spans(&[]), []);
    assert_eq!(spans_hiding(&quiet_with(&[3], 10), &[span(0, 9)]), []);
}
