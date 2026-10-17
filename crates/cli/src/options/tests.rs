use std::num::{NonZeroU32, NonZeroUsize};

use engine::config::DetectConfig;
use engine::settings::DetectSettings;
use render::config::{RenderConfig, Zoom, ZoomConfig};
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

use super::{Change, Options};

#[test]
fn by_default_the_settings_are_those_of_v0_1_1_with_zooms_and_without_annotated_video() {
    let options = Options::default();
    assert_eq!(options.detect, DetectConfig::default());
    assert_eq!(options.track, TrackConfig::default());
    assert_eq!(options.stability, StabilityConfig::default());
    assert_eq!(options.render, RenderConfig::default());
    assert_eq!(options.render.zoom().choice(), Zoom::Auto);
    assert!(!options.annotated);
}

#[test]
fn nothing_is_changed_by_default() {
    assert_eq!(Options::default().changed(), []);
    assert!(!Options::default().tunes_the_frames());
}

#[test]
fn each_setting_of_the_detection_that_differs_from_its_default_is_reported_with_its_value() {
    let detect = DetectSettings {
        threshold: 6.5,
        work_width: 480,
        osd_regions: vec!["0,0,1,0.05".parse().unwrap()],
        ..DetectSettings::default()
    };
    let track = TrackConfig::default();
    let options = Options {
        detect: DetectConfig::try_from(&detect).unwrap(),
        track: TrackConfig::new(
            track.max_jump(),
            track.max_gap(),
            NonZeroUsize::new(3).unwrap(),
            track.min_travel(),
            track.twin_distance(),
            track.max_median_turn(),
        )
        .unwrap(),
        stability: StabilityConfig::new(None, StabilityConfig::default().pad_s()).unwrap(),
        ..Options::default()
    };
    assert_eq!(
        options.changed(),
        [
            Change::OsdRegions(1),
            Change::Threshold(6.5),
            Change::MinHits(NonZeroUsize::new(3).unwrap()),
            Change::MaxBlobs(None),
            Change::WorkWidth(NonZeroU32::new(480).unwrap()),
        ]
    );
    assert!(options.tunes_the_frames());
}

#[test]
fn each_setting_of_the_clips_that_makes_a_video_recalculated_is_reported_in_the_order_of_the_help() {
    let render = RenderConfig::default();
    let options = Options {
        render: RenderConfig::new(render.box_pad(), render.trail_s(), 2.5, 0)
            .unwrap()
            .with_zoom(ZoomConfig::choosing(Zoom::All)),
        annotated: true,
        ..Options::default()
    };
    assert_eq!(
        options.changed(),
        [
            Change::Zoom(Zoom::All),
            Change::Annotated,
            Change::MaxTracks(0),
            Change::ClipMargin(2.5),
        ]
    );
    assert!(!options.tunes_the_frames());
}
