use std::fs;
use std::path::Path;

use serde_json::{Value, json};
use types::frame::{FrameIndex, Pts};
use types::rate::FrameRate;
use types::rational::Rational;
use types::span::Span;
use types::timeline::Timeline;

use super::{Params, spans_in_seconds, write_params};
use crate::error::Error;

fn timeline() -> Timeline {
    let rate = FrameRate::new(Rational::new(10_000, 333).unwrap()).unwrap();
    let pts = (0..9011).map(|frame| Some(Pts::new(frame * 333))).collect();
    Timeline::new(pts, Rational::new(1, 10_000).unwrap(), rate).unwrap()
}

fn settings() -> Value {
    json!({
        "TrackConfig": {"max_jump": 120.0, "max_gap": 6},
        "StabilityConfig": {"max_blobs": 20, "pad_s": 1.0},
    })
}

fn params(settings: &Value) -> Params<'_, Value> {
    Params {
        video: Path::new("/vidéos/nuit_1.mp4"),
        fps: 30.027_388_203_711_72,
        settings,
        ignored_s: vec![[7.0, 8.5]],
        damaged_s: Vec::new(),
        damaged_frames: 0,
        version: "0.2.0",
        ffmpeg: "7.1.1",
    }
}

fn written(params: &Params<'_, Value>) -> (String, Value) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("params.json");
    write_params(&path, params).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    let value = serde_json::from_str(&text).unwrap();
    (text, value)
}

#[test]
fn params_are_written_as_readable_utf8_json_ending_with_a_newline() {
    let (text, _) = written(&params(&settings()));
    assert!(text.contains("/vidéos/nuit_1.mp4"));
    assert!(text.starts_with("{\n  \"video\""));
    assert!(text.ends_with("}\n"));
}

#[test]
fn settings_keep_the_keys_given_by_the_caller_beside_the_run_facts() {
    let (_, value) = written(&params(&settings()));
    assert_eq!(value["TrackConfig"]["max_gap"], 6);
    assert_eq!(value["StabilityConfig"]["pad_s"], 1.0);
    assert_eq!(value["ignored_s"], json!([[7.0, 8.5]]));
    assert_eq!(
        (value["version"].as_str(), value["ffmpeg"].as_str()),
        (Some("0.2.0"), Some("7.1.1"))
    );
}

#[test]
fn neither_the_mode_nor_ffprobe_is_recorded_any_more() {
    let (_, value) = written(&params(&settings()));
    let keys: Vec<&str> = value.as_object().unwrap().keys().map(String::as_str).collect();
    assert!(!keys.contains(&"mode") && !keys.contains(&"ffprobe"));
}

#[test]
fn spans_are_recorded_in_seconds_to_the_hundredth() {
    let spans = [
        Span::new(FrameIndex::new(711), FrameIndex::new(863)).unwrap(),
        Span::new(FrameIndex::new(9000), FrameIndex::new(9010)).unwrap(),
    ];
    assert_eq!(spans_in_seconds(&spans, &timeline()), [[23.68, 28.77], [299.7, 300.07]]);
}

#[test]
fn a_params_file_in_a_missing_folder_is_reported() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("absent").join("params.json");
    assert!(matches!(
        write_params(&path, &params(&settings())).unwrap_err(),
        Error::Write { .. }
    ));
}

#[test]
fn settings_that_are_not_an_object_cannot_be_encoded() {
    let number = json!(3);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("params.json");
    assert!(matches!(
        write_params(&path, &params(&number)).unwrap_err(),
        Error::Encode { .. }
    ));
}
