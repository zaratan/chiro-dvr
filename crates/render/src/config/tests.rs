use types::rate::FrameRate;
use types::rational::Rational;

use super::{Error, RenderConfig};

fn rate(num: i32, den: i32) -> FrameRate {
    FrameRate::new(Rational::new(num, den).unwrap()).unwrap()
}

#[test]
fn the_defaults_are_those_of_v0_1_1() {
    let config = RenderConfig::default();
    assert_eq!(
        (
            config.box_pad(),
            config.trail_s(),
            config.clip_margin_s(),
            config.max_tracks()
        ),
        (12, 1.0, 1.5, 300)
    );
    assert_eq!(RenderConfig::new(12, 1.0, 1.5, 300), Ok(config));
}

#[test]
fn seconds_that_are_negative_infinite_or_not_a_number_are_refused() {
    for value in [-0.5, f64::INFINITY, f64::NAN] {
        assert!(matches!(
            RenderConfig::new(12, value, 1.5, 300),
            Err(Error::Seconds { name: "trail_s", .. })
        ));
        assert!(matches!(
            RenderConfig::new(12, 1.0, value, 300),
            Err(Error::Seconds {
                name: "clip_margin_s",
                ..
            })
        ));
    }
}

#[test]
fn the_binoculars_rate_gives_a_margin_of_45_frames_and_a_trail_of_30() {
    let config = RenderConfig::default();
    let binoculars = rate(90_110_000, 3_000_927);
    assert_eq!(config.margin_frames(binoculars), 45);
    assert_eq!(config.trail_frames(binoculars), 30);
}

#[test]
fn a_margin_halfway_between_two_frames_rounds_up() {
    assert_eq!(RenderConfig::default().margin_frames(rate(27, 1)), 41);
}

#[test]
fn clips_are_rendered_up_to_the_track_limit_and_without_limit_at_0() {
    let config = RenderConfig::default();
    assert!(config.renders_clips_for(300));
    assert!(!config.renders_clips_for(301));
    let unlimited = RenderConfig::new(12, 1.0, 1.5, 0).unwrap();
    assert!(unlimited.renders_clips_for(10_000));
}
