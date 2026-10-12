use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

use compare::agreement::MIN_RATE;
use compare::case::Target;
use compare::comparison::{VIDEOS_AT_ONCE, compare_all, frames_path};
use compare::outcome::Outcome;
use compare::report::{write_refusals, write_report};
use compare::tracks::TrackCounts;
use reference::detection_matching::match_detections;
use reference::frames::Frames;
use reference::location::committed;
use reference::recording::Recording;
use tempfile::TempDir;
use types::detection::FrameDetections;
use types::span::Span;

const DISPLAY_BAND_PX: f64 = 0.07 * 1080.0;

struct Run {
    outcome: Outcome,
    written: PathBuf,
    _folder: TempDir,
}

#[cfg(test)]
fn run() -> &'static Run {
    static RUN: OnceLock<Run> = OnceLock::new();
    RUN.get_or_init(|| {
        let folder = TempDir::new().unwrap();
        let cases = Target::Settings.cases().unwrap();
        let outcome = Outcome::of(&cases, compare_all(&cases, VIDEOS_AT_ONCE, Some(folder.path())));
        let mut stdout = io::stdout().lock();
        write_report(&outcome.compared, &mut stdout).unwrap();
        write_refusals(&outcome.refused, &mut stdout).unwrap();
        Run {
            outcome,
            written: folder.path().to_path_buf(),
            _folder: folder,
        }
    })
}

#[cfg(test)]
fn below_the_display_band(detections: &FrameDetections, ignored: &[Span]) -> FrameDetections {
    let kept = detections.without(ignored);
    FrameDetections::new(
        kept.iter()
            .map(|(_, found)| {
                found
                    .iter()
                    .copied()
                    .filter(|detection| detection.y >= DISPLAY_BAND_PX)
                    .collect()
            })
            .collect(),
    )
    .unwrap()
}

#[test]
#[ignore = "slow"]
fn each_setting_the_engine_cannot_handle_yet_is_refused_for_what_it_needs_and_none_fails() {
    let outcome = &run().outcome;
    let failed: Vec<String> = outcome
        .failed
        .iter()
        .map(|(recording, error)| format!("{}: {error}", recording.display()))
        .collect();
    let refused: Vec<(String, &str)> = outcome
        .refused
        .iter()
        .map(|refusal| {
            let name = refusal.recording.file_name().unwrap().to_string_lossy().into_owned();
            (name, refusal.reason.as_str())
        })
        .collect();
    let expected = [
        ("osd", "osd_regions: masks are not handled"),
        ("step_1", "keeps 31 of 31 window frames, more than 16"),
        ("width_1440_osd", "osd_regions: masks are not handled"),
        ("window_2s", "keeps 21 of 61 window frames, more than 16"),
    ];

    assert_eq!(failed, Vec::<String>::new());
    assert_eq!(
        refused.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>(),
        expected.map(|(name, _)| name)
    );
    for ((name, reason), (_, needs)) in refused.iter().zip(expected) {
        assert!(reason.contains(needs), "{name}: {reason}");
    }
}

#[test]
#[ignore = "slow"]
fn every_setting_the_engine_handles_agrees_with_the_v011_on_its_detections_and_on_every_track() {
    let found: Vec<(&str, bool, TrackCounts)> = run()
        .outcome
        .compared
        .iter()
        .map(|comparison| {
            (
                comparison.detections.name(),
                comparison.detections.holds(),
                comparison.tracks.counts(),
            )
        })
        .collect();
    let agreeing = |tracks: usize| TrackCounts {
        reference: tracks,
        paired: tracks,
        agreeing: tracks,
        lost: 0,
        added: 0,
    };

    assert_eq!(
        found,
        [
            ("max_area_400", true, agreeing(4)),
            ("merge_radius_0", true, agreeing(4)),
            ("min_area_9", true, agreeing(5)),
            ("noise_factor_0", true, agreeing(4)),
            ("quick", true, agreeing(4)),
            ("sigma_0", false, agreeing(30)),
            ("sigma_3", true, agreeing(4)),
            ("size_720x540", true, agreeing(4)),
            ("threshold_25", true, agreeing(4)),
            ("width_1440", true, agreeing(4)),
            ("width_480", true, agreeing(4)),
            ("width_720", true, agreeing(5)),
        ]
    );
}

#[test]
#[ignore = "slow"]
fn without_blur_the_detections_agree_below_the_coloured_display_whose_gray_the_table_does_not_follow() {
    let recording = Recording::read(&committed("settings/sigma_0")).unwrap();
    let rust = Frames::read(&frames_path(&run().written, "sigma_0")).unwrap();
    let summary = &recording.summary;
    let ignored: Vec<Span> = summary
        .damaged_spans
        .iter()
        .chain(&summary.unstable_spans)
        .copied()
        .collect();

    let matching = match_detections(
        &below_the_display_band(recording.frames.detections(), &ignored),
        &below_the_display_band(rust.detections(), &ignored),
    );

    let paired = matching.pairs.len() as f64;
    let reference_rate = paired / (paired + matching.reference_only.len() as f64);
    let rust_rate = paired / (paired + matching.candidate_only.len() as f64);
    assert!(
        reference_rate >= MIN_RATE && rust_rate >= MIN_RATE,
        "{reference_rate} {rust_rate}"
    );
}
