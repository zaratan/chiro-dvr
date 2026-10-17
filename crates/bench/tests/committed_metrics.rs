use bench::folder;
use bench::metrics::{Judging, evaluate};
use bench::report::{Seconds, bench_report};
use reference::bench::{BenchReport, REPORT_FILE, read_report};
use reference::location::committed_bench;
use serde::Serialize;
use serde_json::Value;

const VIDEO_FOLDER: &str = "video_092_original";
const FLOAT_TOLERANCE: f64 = 1e-9;
const DURATIONS: &str = "seconds";
const REJOIN_HIDDEN_SINCE_V020: usize = 19;
const REJOIN_DISTANCE_SINCE_V020: f64 = 60.0;
const REJOIN_PIECE_POINTS_SINCE_V020: usize = 2;
const REJOIN_JUNCTION_PACE_SINCE_V020: f64 = 0.25;
const REJOIN_PIECE_PACE_SINCE_V020: f64 = 0.45;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Metrics {
    not_visible: usize,
    false_tracks: usize,
    reference_tracks: usize,
    injected_tracks: usize,
    evaluated: usize,
    found: usize,
    targets_changed: usize,
    classes_changed: usize,
}

const FROZEN: [(&str, Metrics); 2] = [
    (
        "transit",
        Metrics {
            not_visible: 17,
            false_tracks: 0,
            reference_tracks: 14,
            injected_tracks: 162,
            evaluated: 253,
            found: 148,
            targets_changed: 2,
            classes_changed: 2,
        },
    ),
    (
        "hunt",
        Metrics {
            not_visible: 131,
            false_tracks: 0,
            reference_tracks: 14,
            injected_tracks: 499,
            evaluated: 589,
            found: 455,
            targets_changed: 26,
            classes_changed: 10,
        },
    ),
];

#[cfg(test)]
fn gaps(found: &Value, recorded: &Value, at: &str, out: &mut Vec<String>) {
    match (found, recorded) {
        (Value::Object(found), Value::Object(recorded)) if found.keys().eq(recorded.keys()) => {
            for (key, value) in found {
                gaps(value, &recorded[key], &format!("{at}.{key}"), out);
            }
        }
        (Value::Array(found), Value::Array(recorded)) if found.len() == recorded.len() => {
            for (index, (found, recorded)) in found.iter().zip(recorded).enumerate() {
                gaps(found, recorded, &format!("{at}[{index}]"), out);
            }
        }
        (Value::Number(found), Value::Number(recorded)) if found.is_f64() || recorded.is_f64() => {
            let (found, recorded) = (found.as_f64().unwrap(), recorded.as_f64().unwrap());
            if (found - recorded).abs() > FLOAT_TOLERANCE {
                out.push(format!("{at}: {found} for {recorded}"));
            }
        }
        _ if found != recorded => out.push(format!("{at}: {found} for {recorded}")),
        _ => {}
    }
}

#[cfg(test)]
fn changed<T: Serialize>(name: &str, found: &[T], recorded: &[T]) -> usize {
    let lines: Vec<Vec<String>> = found
        .iter()
        .zip(recorded)
        .enumerate()
        .map(|(index, (found, recorded))| {
            let mut out = Vec::new();
            gaps(
                &serde_json::to_value(found).unwrap(),
                &serde_json::to_value(recorded).unwrap(),
                &format!("{name}[{index}]"),
                &mut out,
            );
            out
        })
        .collect();
    for line in lines.iter().flatten() {
        println!("{line}");
    }
    lines.iter().filter(|out| !out.is_empty()).count() + found.len().abs_diff(recorded.len())
}

#[cfg(test)]
fn settings(report: &BenchReport) -> Value {
    let mut added = report.added.clone();
    added.remove(DURATIONS);
    serde_json::json!({
        "video": report.video,
        "code": report.code,
        "detect": report.detect,
        "track": report.track,
        "stability": report.stability,
        "match": report.matching,
        "sampling": report.sampling,
        "added": added,
    })
}

#[cfg(test)]
fn judged_now(recorded: &BenchReport) -> Value {
    let mut settings = settings(recorded);
    settings["track"]["rejoin_hidden"] = serde_json::json!(REJOIN_HIDDEN_SINCE_V020);
    settings["track"]["rejoin_distance"] = serde_json::json!(REJOIN_DISTANCE_SINCE_V020);
    settings["track"]["rejoin_piece_points"] = serde_json::json!(REJOIN_PIECE_POINTS_SINCE_V020);
    settings["track"]["rejoin_junction_pace"] = serde_json::json!(REJOIN_JUNCTION_PACE_SINCE_V020);
    settings["track"]["rejoin_piece_pace"] = serde_json::json!(REJOIN_PIECE_PACE_SINCE_V020);
    settings
}

#[test]
fn each_v020_bench_evaluated_again_with_the_default_judging_keeps_its_settings_and_its_frozen_metrics() {
    for (name, frozen) in FROZEN {
        let folder = committed_bench(name).join(VIDEO_FOLDER);
        let run = folder::read(&folder).unwrap();
        let recorded = read_report(&folder.join(REPORT_FILE)).unwrap();

        let evaluation = evaluate(&run, Judging::default()).unwrap();

        let found = bench_report(
            &run,
            Judging::default(),
            &evaluation,
            recorded.code.clone(),
            Seconds::default(),
        )
        .unwrap();
        assert_eq!(settings(&found), judged_now(&recorded), "{name}");
        let metrics = Metrics {
            not_visible: found.not_visible,
            false_tracks: found.false_tracks,
            reference_tracks: found.reference_tracks,
            injected_tracks: found.injected_tracks,
            evaluated: found.bats.len(),
            found: found.bats.iter().filter(|target| target.found).count(),
            targets_changed: changed(&format!("{name} target"), &found.bats, &recorded.bats),
            classes_changed: changed(&format!("{name} class"), &found.summary, &recorded.summary),
        };
        assert_eq!(metrics, frozen, "{name}");
    }
}
