use std::fs;
use std::path::Path;

use engine::config::DetectConfig;
use engine::settings::{DetectSettings, Region};
use files::names::params_json;
use files::params::{Finish, Params, language_code, local_date, write_params};
use jiff::Zoned;
use media::decode::ActiveThreads;
use media::encode::Codec;
use serde_json::{Value, json};
use types::language::Language;

use super::{Earlier, Run, earlier};
use crate::jobs::Job;
use crate::options::Options;
use crate::process::Extent;
use crate::settings::Settings;

type Edit = fn(&mut Value);

const VIDEO_BYTES: usize = 2510;
const VERSION: &str = "0.1.1-160-g2dbb732";

fn finished(folder: &Path, options: &Options, version: &str, edit: impl FnOnce(&mut Value)) -> Job {
    let job = Job {
        video: folder.join("flight.mp4"),
        dest: folder.join("out").join("flight"),
    };
    fs::write(&job.video, vec![0_u8; VIDEO_BYTES]).unwrap();
    fs::create_dir_all(&job.dest).unwrap();
    let settings = Settings::of(options, &[Codec::OpenH264], ActiveThreads::Single, 4);
    let params = Params {
        video: &job.video,
        fps: 30.0,
        settings: &settings,
        ignored_s: Vec::new(),
        damaged_s: Vec::new(),
        damaged_frames: 0,
        version,
        ffmpeg: "7.1.1",
        state: Finish::Done,
        track_count: 4,
        length_s: 40.959,
        not_analysed_s: 0.5,
        elapsed_s: 4.4,
        video_bytes: VIDEO_BYTES as u64,
        language: language_code(Language::French),
        date: local_date(&Zoned::now()),
        background: None,
        summaries: Vec::new(),
        tracks: Vec::new(),
    };
    let path = params_json(&job.dest);
    write_params(&path, &params).unwrap();
    let mut value: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    edit(&mut value);
    fs::write(&path, value.to_string()).unwrap();
    job
}

fn found(options: &Options, edit: impl FnOnce(&mut Value)) -> Option<Earlier> {
    found_after(options, options, VERSION, edit)
}

fn found_after(written: &Options, options: &Options, version: &str, edit: impl FnOnce(&mut Value)) -> Option<Earlier> {
    let folder = tempfile::tempdir().unwrap();
    let job = finished(folder.path(), written, version, edit);
    earlier(
        &job,
        Run {
            options,
            version,
            language: Language::French,
        },
    )
}

fn leaves(value: &Value, pointer: &str, found: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, inner) in map {
                leaves(inner, &format!("{pointer}/{key}"), found);
            }
        }
        _ => found.push(pointer.to_owned()),
    }
}

#[test]
fn the_same_command_on_the_same_video_finds_the_earlier_result_with_its_figures() {
    assert_eq!(
        found(&Options::default(), |_| {}),
        Some(Earlier {
            finish: Finish::Done,
            tracks: 4,
            limit: 300,
            extent: Extent {
                length_s: 40.959,
                not_analysed_s: 0.5,
            },
        })
    );
}

#[test]
fn a_video_left_with_too_many_tracks_is_found_with_that_state() {
    let earlier = found(&Options::default(), |value| value["state"] = json!("too_many_tracks"));
    assert_eq!(earlier.map(|earlier| earlier.finish), Some(Finish::TooManyTracks));
}

#[test]
fn the_machine_the_encoders_and_the_results_do_not_make_a_video_be_redone() {
    let earlier = found(&Options::default(), |value| {
        value["RenderConfig"]["encoders"] = json!(["videotoolbox", "openh264"]);
        value["threads"] = json!({"decoding": 11, "decoding_kind": "frame", "detection": 10});
        value["fps"] = json!(29.97);
        value["ffmpeg"] = json!("7.1.2");
        value["video"] = json!("/ailleurs/flight.mp4");
        value["date"] = json!("2026-10-17T21:00:00+02:00");
        value["elapsed_s"] = json!(99.0);
        value["tracks"] = json!([{"id": 1}]);
    });
    assert!(earlier.is_some());
}

#[test]
fn another_setting_version_size_or_language_makes_the_video_be_redone() {
    let edits: [(&str, Edit); 6] = [
        ("threshold", |value| value["DetectConfig"]["threshold"] = json!(13.0)),
        ("min hits", |value| value["TrackConfig"]["min_hits"] = json!(5)),
        ("annotated", |value| value["RenderConfig"]["annotated"] = json!(true)),
        ("dirty", |value| value["version"] = json!(format!("{VERSION}-dirty"))),
        ("size", |value| value["video_bytes"] = json!(VIDEO_BYTES + 1)),
        ("language", |value| value["language"] = json!("en")),
    ];
    for (name, edit) in edits {
        assert_eq!(found(&Options::default(), edit), None, "{name}");
    }
}

#[test]
fn options_given_now_that_differ_from_the_earlier_run_make_the_video_be_redone() {
    let annotated = Options {
        annotated: true,
        ..Options::default()
    };
    assert_eq!(found_after(&Options::default(), &annotated, VERSION, |_| {}), None);
}

#[test]
fn a_folder_without_params_json_or_with_one_from_before_the_state_is_redone() {
    assert_eq!(
        found(&Options::default(), |value| {
            value.as_object_mut().unwrap().remove("state");
        }),
        None
    );
    let folder = tempfile::tempdir().unwrap();
    let job = finished(folder.path(), &Options::default(), VERSION, |_| {});
    fs::remove_file(params_json(&job.dest)).unwrap();
    let run = Run {
        options: &Options::default(),
        version: VERSION,
        language: Language::French,
    };
    assert_eq!(earlier(&job, run), None);
}

#[test]
fn a_video_gone_since_the_earlier_run_is_not_reported_as_already_done() {
    let folder = tempfile::tempdir().unwrap();
    let job = finished(folder.path(), &Options::default(), VERSION, |_| {});
    fs::remove_file(&job.video).unwrap();
    let run = Run {
        options: &Options::default(),
        version: VERSION,
        language: Language::French,
    };
    assert_eq!(earlier(&job, run), None);
}

#[test]
fn a_build_with_uncommitted_changes_never_takes_an_earlier_result_even_its_own() {
    let dirty = format!("{VERSION}-dirty");
    assert_eq!(
        found_after(&Options::default(), &Options::default(), &dirty, |_| {}),
        None
    );
}

#[test]
fn every_setting_written_in_params_json_but_the_threads_and_the_encoders_is_compared() {
    let settings = Settings::of(&Options::default(), &[Codec::OpenH264], ActiveThreads::Single, 4);
    let mut pointers = Vec::new();
    leaves(&serde_json::to_value(settings).unwrap(), "", &mut pointers);
    let compared: Vec<&String> = pointers
        .iter()
        .filter(|pointer| !pointer.starts_with("/threads") && !pointer.ends_with("/encoders"))
        .collect();
    assert!(compared.len() > 20, "{compared:?}");
    for pointer in compared {
        let earlier = found(&Options::default(), |value| {
            *value.pointer_mut(pointer).unwrap() = json!("changed");
        });
        assert_eq!(earlier, None, "{pointer}");
    }
}

#[test]
fn the_same_settings_with_long_decimals_read_back_from_params_json_still_find_the_earlier_result() {
    let settings = DetectSettings {
        threshold: 12.345_678_901_234_567,
        osd_regions: vec![Region {
            x0: 0.0,
            y0: 1.0 / 3.0,
            x1: 2.0 / 3.0,
            y1: 0.999_999_999_999_999_9,
        }],
        ..DetectSettings::default()
    };
    let options = Options {
        detect: DetectConfig::try_from(&settings).unwrap(),
        ..Options::default()
    };
    assert!(found(&options, |_| {}).is_some());
}
