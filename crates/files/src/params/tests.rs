use std::fs;
use std::path::Path;

use jiff::Zoned;
use serde_json::{Value, json};
use types::frame::{FrameIndex, Pts};
use types::language::Language;
use types::rate::FrameRate;
use types::rational::Rational;
use types::span::Span;
use types::timeline::Timeline;

use super::{
    Finish, Params, Recorded, TrackEntry, language_code, local_date, read_params, spans_in_seconds, write_params,
};
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

fn evening() -> Zoned {
    "2026-10-16T21:04:33.75+02:00[+02:00]".parse().unwrap()
}

fn params<'a>(settings: &'a Value, date: &Zoned) -> Params<'a, Value> {
    Params {
        video: Path::new("/vidéos/nuit_1.mp4"),
        fps: 30.027_388_203_711_72,
        settings,
        ignored_s: vec![[7.0, 8.5]],
        damaged_s: Vec::new(),
        damaged_frames: 0,
        version: "0.2.0",
        ffmpeg: "7.1.1",
        state: Finish::Done,
        track_count: 2,
        length_s: 300.067,
        not_analysed_s: 2.5,
        elapsed_s: 18.734_5,
        video_bytes: 1_073_741_824,
        language: language_code(Language::French),
        date: local_date(date),
        tracks: vec![
            TrackEntry {
                id: 1,
                start_s: 4.137_5,
                duration_s: 0.033_3,
                color: Some([255, 214, 10]),
                summary: Some("nuit_1_resume_3_sur_8.png".to_owned()),
                clip: Some("extraits/piste_01_0m04s.mp4".to_owned()),
                zoom: Some("extraits/piste_01_0m04s_zoom.mp4".to_owned()),
            },
            TrackEntry {
                id: 2,
                start_s: 209.5,
                duration_s: 1.25,
                color: None,
                summary: None,
                clip: None,
                zoom: None,
            },
        ],
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
    let (text, _) = written(&params(&settings(), &evening()));
    assert!(text.contains("/vidéos/nuit_1.mp4"));
    assert!(text.starts_with("{\n  \"video\""));
    assert!(text.ends_with("}\n"));
}

#[test]
fn settings_keep_the_keys_given_by_the_caller_beside_the_run_facts() {
    let (_, value) = written(&params(&settings(), &evening()));
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
    let (_, value) = written(&params(&settings(), &evening()));
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
        write_params(&path, &params(&settings(), &evening())).unwrap_err(),
        Error::Write { .. }
    ));
}

#[test]
fn settings_that_are_not_an_object_cannot_be_encoded() {
    let number = json!(3);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("params.json");
    assert!(matches!(
        write_params(&path, &params(&number, &evening())).unwrap_err(),
        Error::Encode { .. }
    ));
}

#[test]
fn the_state_is_written_as_an_untranslated_word() {
    let settings = settings();
    let date = evening();
    let words = [Finish::Done, Finish::TooManyTracks].map(|state| {
        let (_, value) = written(&Params {
            state,
            ..params(&settings, &date)
        });
        value["state"].clone()
    });
    assert_eq!(words, [json!("done"), json!("too_many_tracks")]);
}

#[test]
fn the_figures_of_the_board_and_the_size_of_the_video_are_recorded_as_they_were_measured() {
    let (_, value) = written(&params(&settings(), &evening()));
    assert_eq!(value["track_count"], 2);
    assert_eq!(value["length_s"], 300.067);
    assert_eq!(value["not_analysed_s"], 2.5);
    assert_eq!(value["elapsed_s"], 18.734_5);
    assert_eq!(value["video_bytes"], 1_073_741_824_u64);
}

#[test]
fn the_language_of_the_names_is_written_as_its_two_letter_code() {
    let settings = settings();
    let date = evening();
    let codes = [Language::French, Language::English].map(|language| {
        let (_, value) = written(&Params {
            language: language_code(language),
            ..params(&settings, &date)
        });
        value["language"].clone()
    });
    assert_eq!(codes, [json!("fr"), json!("en")]);
}

#[test]
fn the_date_is_the_local_time_to_the_second_with_its_offset() {
    let (_, value) = written(&params(&settings(), &evening()));
    assert_eq!(value["date"], "2026-10-16T21:04:33+02:00");
}

#[test]
fn an_earlier_params_file_is_replaced_whole_and_no_partial_file_is_left() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("params.json");
    fs::write(&path, "x".repeat(10_000)).unwrap();
    write_params(&path, &params(&settings(), &evening())).unwrap();
    let text = fs::read_to_string(&path).unwrap();
    assert!(text.starts_with('{') && text.ends_with("}\n"), "{text}");
    let mut names: Vec<String> = fs::read_dir(directory.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    assert_eq!(names, ["params.json"]);
}

#[test]
fn each_track_is_listed_with_its_exact_times_its_colour_its_summary_image_and_the_names_of_its_clips_or_null() {
    let (_, value) = written(&params(&settings(), &evening()));
    assert_eq!(
        value["tracks"],
        json!([
            {
                "id": 1,
                "start_s": 4.137_5,
                "duration_s": 0.033_3,
                "color": [255, 214, 10],
                "summary": "nuit_1_resume_3_sur_8.png",
                "clip": "extraits/piste_01_0m04s.mp4",
                "zoom": "extraits/piste_01_0m04s_zoom.mp4"
            },
            {"id": 2, "start_s": 209.5, "duration_s": 1.25, "color": null, "summary": null, "clip": null, "zoom": null}
        ])
    );
}

fn read_back(edit: impl FnOnce(&mut Value)) -> Option<Recorded> {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("params.json");
    let (_, mut value) = written(&params(&settings(), &evening()));
    edit(&mut value);
    fs::write(&path, value.to_string()).unwrap();
    read_params(&path)
}

#[test]
fn a_params_file_written_by_this_version_is_read_back_with_its_state_its_figures_and_its_settings() {
    let recorded = read_back(|_| {}).unwrap();
    assert_eq!(recorded.state, Finish::Done);
    assert_eq!(
        (
            recorded.version.as_str(),
            recorded.video_bytes,
            recorded.language.as_str()
        ),
        ("0.2.0", 1_073_741_824, "fr")
    );
    assert_eq!(
        (recorded.track_count, recorded.length_s, recorded.not_analysed_s),
        (2, 300.067, 2.5)
    );
    assert_eq!(recorded.rest["TrackConfig"]["max_gap"], 6);
}

#[test]
fn a_params_file_without_state_from_an_earlier_version_reads_as_unfinished() {
    let recorded = read_back(|value| {
        value.as_object_mut().unwrap().remove("state");
    });
    assert_eq!(recorded, None);
}

#[test]
fn a_params_file_with_an_unknown_state_cut_short_or_absent_reads_as_unfinished() {
    assert_eq!(read_back(|value| value["state"] = json!("started")), None);
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("params.json");
    assert_eq!(read_params(&path), None);
    fs::write(&path, "{\"state\": \"done\", \"vers").unwrap();
    assert_eq!(read_params(&path), None);
}
