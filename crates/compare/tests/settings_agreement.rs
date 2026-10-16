use std::io;
use std::path::PathBuf;
use std::sync::OnceLock;

use compare::agreement::MIN_RATE;
use compare::case::Target;
use compare::comparison::{VIDEOS_AT_ONCE, compare_all, frames_path};
use compare::outcome::Outcome;
use compare::report::write_report;
use compare::tracks::TrackCounts;
use reference::detection_matching::match_detections;
use reference::frames::Frames;
use reference::location::committed;
use reference::recording::Recording;
use tempfile::TempDir;
use types::detection::{FrameDetections, Measures};
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
        let folder = TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
        let cases = Target::Settings.cases().unwrap();
        let outcome = Outcome::of(&cases, compare_all(&cases, VIDEOS_AT_ONCE, Some(folder.path())));
        let mut stdout = io::stdout().lock();
        write_report(&outcome.compared, &mut stdout).unwrap();
        Run {
            outcome,
            written: folder.path().to_path_buf(),
            _folder: folder,
        }
    })
}

fn below_the_display_band(detections: &FrameDetections, ignored: &[Span]) -> FrameDetections {
    let kept = detections.without(ignored);
    let mut below = FrameDetections::default();
    for (_, found) in kept.iter() {
        below.push(
            found
                .iter()
                .filter(|detection| detection.y >= DISPLAY_BAND_PX)
                .map(Measures::of),
        );
    }
    below
}

#[test]
#[ignore = "settings"]
fn every_setting_of_the_bench_is_compared_and_none_fails() {
    let outcome = &run().outcome;
    let failed: Vec<String> = outcome
        .failed
        .iter()
        .map(|(recording, error)| format!("{}: {error}", recording.display()))
        .collect();

    assert_eq!(failed, Vec::<String>::new());
    assert_eq!(outcome.compared.len(), 16);
}

#[test]
#[ignore = "settings"]
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
            ("osd", true, agreeing(4)),
            ("quick", true, agreeing(4)),
            ("sigma_0", false, agreeing(30)),
            ("sigma_3", true, agreeing(4)),
            ("size_720x540", true, agreeing(4)),
            ("step_1", true, agreeing(5)),
            ("threshold_25", true, agreeing(4)),
            ("width_1440", true, agreeing(4)),
            ("width_1440_osd", true, agreeing(4)),
            ("width_480", true, agreeing(4)),
            ("width_720", true, agreeing(5)),
            ("window_2s", false, agreeing(4)),
        ]
    );
}

#[test]
#[ignore = "settings"]
fn sigma_0_and_window_2s_agree_below_the_coloured_display_whose_gray_the_table_does_not_follow() {
    for case in ["sigma_0", "window_2s"] {
        agrees_below_the_display_band(case);
    }
}

#[cfg(test)]
fn agrees_below_the_display_band(case: &str) {
    let recording = Recording::read(&committed(&format!("settings/{case}"))).unwrap();
    let rust = Frames::read(&frames_path(&run().written, case)).unwrap();
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
    println!("{case} below the display band: {reference_rate:.4} {rust_rate:.4}");
    assert!(
        reference_rate >= MIN_RATE && rust_rate >= MIN_RATE,
        "{case}: {reference_rate} {rust_rate}"
    );
}
