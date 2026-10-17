use std::num::NonZeroUsize;

use super::{StabilitySettings, TrackSettings};
use crate::stability::StabilityConfig;
use crate::track::TrackConfig;

#[test]
fn the_default_settings_are_written_as_the_v011_wrote_them_then_the_rejoin_and_its_pieces() {
    assert_eq!(
        serde_json::to_string(&TrackSettings::from(TrackConfig::default())).unwrap(),
        r#"{"max_jump":120.0,"max_gap":6,"min_hits":6,"min_travel":45.0,"twin_distance":36.0,"max_median_turn":0.8,"rejoin_hidden":19,"rejoin_distance":60.0,"rejoin_piece_points":2,"rejoin_junction_pace":0.25,"rejoin_piece_pace":0.45}"#
    );
    assert_eq!(
        serde_json::to_string(&StabilitySettings::from(StabilityConfig::default())).unwrap(),
        r#"{"max_blobs":20,"pad_s":1.0}"#
    );
}

#[test]
fn disabled_filters_are_written_as_the_zero_of_v011() {
    let track = TrackConfig::new(120.0, NonZeroUsize::MIN, NonZeroUsize::MIN, 45.0, None, 0.8, None).unwrap();
    let stability = StabilityConfig::new(None, 1.0).unwrap();

    assert_eq!(TrackSettings::from(track).twin_distance, 0.0);
    assert_eq!(
        (
            TrackSettings::from(track).rejoin_hidden,
            TrackSettings::from(track).rejoin_distance,
            TrackSettings::from(track).rejoin_piece_points,
            TrackSettings::from(track).rejoin_junction_pace,
            TrackSettings::from(track).rejoin_piece_pace,
        ),
        (0, 0.0, 0, 0.0, 0.0)
    );
    assert_eq!(StabilitySettings::from(stability).max_blobs, 0);
}

#[test]
fn settings_written_by_the_v011_read_back_whole_numbers_included_and_without_rejoin() {
    let track: TrackSettings = serde_json::from_str(
        r#"{"max_jump": 120, "max_gap": 6, "min_hits": 6, "min_travel": 45, "twin_distance": 36, "max_median_turn": 0.8}"#,
    )
    .unwrap();

    assert_eq!(track, TrackSettings::from(TrackConfig::default().without_rejoin()));
    assert!(serde_json::from_str::<StabilitySettings>(r#"{"max_blobs": 20, "pad_s": 1.0, "window": 3}"#).is_err());
}
