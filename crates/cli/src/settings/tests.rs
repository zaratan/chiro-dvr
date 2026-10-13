use std::num::NonZeroUsize;

use engine::config::DetectConfig;
use engine::settings::{DetectSettings, Region};
use media::decode::ActiveThreads;
use media::encode::Codec;
use render::config::RenderConfig;
use serde_json::{Value, json};
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

use super::Settings;

fn written(track: TrackConfig, stability: StabilityConfig, decoding: ActiveThreads) -> Value {
    let settings = Settings::of(
        &DetectConfig::default(),
        track,
        stability,
        RenderConfig::default(),
        &[Codec::VideoToolbox, Codec::OpenH264],
        decoding,
        10,
    );
    serde_json::to_value(settings).unwrap()
}

#[test]
fn configs_are_keyed_by_their_v0_1_1_class_name() {
    let value = written(
        TrackConfig::default(),
        StabilityConfig::default(),
        ActiveThreads::Single,
    );
    let keys: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        [
            "DetectConfig",
            "RenderConfig",
            "StabilityConfig",
            "TrackConfig",
            "threads"
        ]
    );
    assert_eq!(value["DetectConfig"]["threshold"], 12.0);
    assert_eq!(value["DetectConfig"]["bg_step"], 3);
    assert_eq!(value["DetectConfig"]["work_width"], 960);
    assert_eq!(value["TrackConfig"]["twin_distance"], 36.0);
    assert_eq!(value["StabilityConfig"], json!({"max_blobs": 20, "pad_s": 1.0}));
}

#[test]
fn disabled_filters_are_written_as_the_zero_of_v0_1_1() {
    let track = TrackConfig::new(120.0, NonZeroUsize::MIN, NonZeroUsize::MIN, 45.0, None, 0.8).unwrap();
    let stability = StabilityConfig::new(None, 1.0).unwrap();
    let value = written(track, stability, ActiveThreads::Single);
    assert_eq!(value["TrackConfig"]["twin_distance"], 0.0);
    assert_eq!(value["StabilityConfig"]["max_blobs"], 0);
}

#[test]
fn the_threads_really_used_are_recorded_with_their_kind() {
    let eight = NonZeroUsize::new(8).unwrap();
    let threads = |decoding| written(TrackConfig::default(), StabilityConfig::default(), decoding)["threads"].clone();
    assert_eq!(
        threads(ActiveThreads::Frame(eight)),
        json!({"decoding": 8, "decoding_kind": "frame", "detection": 10})
    );
    assert_eq!(threads(ActiveThreads::Slice(eight))["decoding_kind"], "slice");
    assert_eq!(threads(ActiveThreads::Single)["decoding"], 1);
}

#[test]
fn the_detection_settings_are_written_in_the_order_and_shape_of_the_v011_osd_regions_included() {
    let settings = DetectSettings {
        osd_regions: vec![Region {
            x0: 0.0,
            y0: 0.0,
            x1: 1.0,
            y1: 0.07,
        }],
        ..DetectSettings::default()
    };

    let written = serde_json::to_string(&settings).unwrap();

    assert_eq!(
        written,
        concat!(
            r#"{"threshold":12.0,"min_area":4.0,"max_area":2700.0,"bg_window_s":1.0,"bg_step":3,"#,
            r#""osd_regions":[{"x0":0.0,"y0":0.0,"x1":1.0,"y1":0.07}],"merge_radius":6.0,"work_width":960,"#,
            r#""noise_factor":8.0,"target_sigma":1.5}"#
        )
    );
    assert_eq!(written_by_default()["DetectConfig"]["osd_regions"], json!([]));
}

fn written_by_default() -> Value {
    written(
        TrackConfig::default(),
        StabilityConfig::default(),
        ActiveThreads::Single,
    )
}

#[test]
fn the_clip_settings_name_the_encoders_tried_in_their_fallback_order() {
    let value = written(
        TrackConfig::default(),
        StabilityConfig::default(),
        ActiveThreads::Single,
    );
    assert_eq!(
        value["RenderConfig"],
        json!({
            "box_pad": 12,
            "trail_s": 1.0,
            "clip_margin_s": 1.5,
            "max_tracks": 300,
            "encoders": ["videotoolbox", "openh264"],
        })
    );
}
