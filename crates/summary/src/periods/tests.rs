use types::rational::Rational;

use super::{SPAN_SECONDS, split_by_period, video_seconds};
use crate::testing::{timeline, timeline_at, track};

const FPS: f64 = 10.0;

fn starting_at(id: usize, seconds: f64) -> types::track::Track {
    track(
        id,
        (seconds * FPS).round() as usize,
        &[[10.0, 10.0], [11.0, 10.0], [12.0, 10.0]],
    )
}

fn lasting(seconds: f64) -> types::timeline::Timeline {
    timeline((seconds * FPS).round() as usize)
}

#[test]
fn short_video_gives_a_single_image_without_suffix() {
    let tracks = [starting_at(1, 5.0)];
    let periods = split_by_period(&tracks, &lasting(300.0), &[], &[]);
    assert_eq!(periods.len(), 1);
    assert_eq!(
        (periods[0].start, periods[0].end, periods[0].minutes()),
        (0.0, 300.0, None)
    );
    assert_eq!(periods[0].tracks.len(), 1);
}

#[test]
fn video_exactly_one_span_long_still_gives_a_single_image() {
    assert_eq!(split_by_period(&[], &lasting(SPAN_SECONDS), &[], &[]).len(), 1);
}

#[test]
fn long_video_is_split_and_the_last_period_ends_with_the_video() {
    let periods = split_by_period(&[], &lasting(1400.0), &[], &[]);
    let bounds: Vec<(f64, f64)> = periods.iter().map(|period| (period.start, period.end)).collect();
    assert_eq!(bounds, [(0.0, 600.0), (600.0, 1200.0), (1200.0, 1400.0)]);
    let minutes: Vec<_> = periods.iter().map(super::Period::minutes).collect();
    assert_eq!(minutes, [Some(0..10), Some(10..20), Some(20..24)]);
}

#[test]
fn track_goes_to_the_period_where_it_starts_even_on_the_boundary() {
    let tracks = [starting_at(1, 599.9), starting_at(2, 600.0)];
    let periods = split_by_period(&tracks, &lasting(1400.0), &[], &[]);
    let ids: Vec<Vec<usize>> = periods
        .iter()
        .map(|period| period.tracks.iter().map(|track| track.id()).collect())
        .collect();
    assert_eq!(ids, [vec![1], vec![2], vec![]]);
}

#[test]
fn ignored_time_goes_to_the_periods_it_touches_clipped_to_their_bounds() {
    let periods = split_by_period(&[], &lasting(1500.0), &[[30.0, 40.0], [590.0, 610.0]], &[]);
    let ignored: Vec<_> = periods.iter().map(|period| period.ignored.clone()).collect();
    assert_eq!(
        ignored,
        [vec![[30.0, 40.0], [590.0, 600.0]], vec![[600.0, 610.0]], vec![]]
    );
}

#[test]
fn damaged_span_across_two_periods_counts_in_both() {
    let periods = split_by_period(&[], &lasting(1500.0), &[], &[[595.0, 605.0]]);
    let damaged: Vec<_> = periods.iter().map(|period| period.damaged.clone()).collect();
    assert_eq!(damaged, [vec![[595.0, 600.0]], vec![[600.0, 605.0]], vec![]]);
}

#[test]
fn the_concatenated_video_of_27_033_frames_gives_two_images_named_by_their_minutes() {
    let concatenated = timeline_at(Rational::new(10_000, 333).unwrap(), 27_033);
    let periods = split_by_period(&[], &concatenated, &[], &[]);
    assert!((video_seconds(&concatenated) - 900.1989).abs() < 1e-6);
    let minutes: Vec<_> = periods.iter().map(super::Period::minutes).collect();
    assert_eq!(minutes, [Some(0..10), Some(10..16)]);
}

#[test]
fn the_duration_counts_the_last_frame_to_its_end() {
    assert!((video_seconds(&timeline(3)) - 0.3).abs() < 1e-12);
}
