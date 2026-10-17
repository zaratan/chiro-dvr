use types::detection::{Detection, Strength};
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::rational::Rational;
use types::track::Track;

use super::{Error, RenderConfig, Seconds, Zoom, ZoomConfig};

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
            Err(Error::Seconds {
                setting: Seconds::Trail,
                ..
            })
        ));
        assert!(matches!(
            RenderConfig::new(12, 1.0, value, 300),
            Err(Error::Seconds {
                setting: Seconds::ClipMargin,
                ..
            })
        ));
    }
}

#[test]
fn a_refused_number_of_seconds_is_named_as_params_json_names_it() {
    let refused = |trail_s, clip_margin_s| {
        RenderConfig::new(12, trail_s, clip_margin_s, 300)
            .unwrap_err()
            .to_string()
    };
    assert_eq!(
        refused(-1.0, 1.5),
        "trail_s must be a finite number of seconds, at least 0, not -1"
    );
    assert_eq!(
        refused(1.0, -2.0),
        "clip_margin_s must be a finite number of seconds, at least 0, not -2"
    );
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

fn measured(area: f64, amplitude: f64) -> Track {
    let points = (0..3)
        .map(|frame| Detection {
            frame: FrameIndex::new(frame),
            x: 50.0,
            y: 50.0,
            left: 45.0,
            top: 45.0,
            width: 10.0,
            height: 10.0,
            area,
            amplitude,
            strength: Strength::Strong,
        })
        .collect();
    Track::new(1, points).unwrap()
}

#[test]
fn auto_zooms_a_track_strictly_below_either_threshold_of_v0_1_1() {
    let auto = ZoomConfig::default();
    assert_eq!(
        (auto.choice(), auto.below_area(), auto.below_amplitude()),
        (Zoom::Auto, 100.0, 40.0)
    );
    assert!(auto.wants(&measured(99.0, 60.0)));
    assert!(!auto.wants(&measured(101.25, 60.0)));
    assert!(!auto.wants(&measured(100.0, 60.0)));
    assert!(auto.wants(&measured(200.0, 39.9)));
    assert!(!auto.wants(&measured(200.0, 40.0)));
}

#[test]
fn all_zooms_every_track_and_none_no_track() {
    let large = measured(500.0, 90.0);
    assert!(ZoomConfig::choosing(Zoom::All).wants(&large));
    assert!(!ZoomConfig::choosing(Zoom::Off).wants(&measured(4.5, 10.0)));
}

#[test]
fn zoom_thresholds_must_be_finite_and_above_zero() {
    for value in [0.0, -1.0, f64::INFINITY, f64::NAN] {
        assert!(matches!(
            ZoomConfig::new(Zoom::Auto, value, 40.0),
            Err(Error::Threshold {
                name: "zoom_below_area",
                ..
            })
        ));
        assert!(matches!(
            ZoomConfig::new(Zoom::Auto, 100.0, value),
            Err(Error::Threshold {
                name: "zoom_below_amplitude",
                ..
            })
        ));
    }
}

#[test]
fn the_zoom_choices_are_named_as_the_command_takes_them() {
    let names: Vec<&str> = Zoom::CHOICES.iter().map(|choice| choice.name()).collect();
    assert_eq!(names, ["auto", "all", "none"]);
    let config = RenderConfig::default().with_zoom(ZoomConfig::choosing(Zoom::All));
    assert_eq!(config.zoom().choice(), Zoom::All);
    assert_eq!(RenderConfig::default().zoom(), ZoomConfig::default());
}
