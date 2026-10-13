use std::num::{NonZeroU32, NonZeroUsize};

use bench::metrics::{Evaluation, Judging, evaluate};
use bench::recorded;
use bench::run::{Provenance, Run, Video};
use bench::stats::{ClassSummary, summarize};
use engine::settings::DetectSettings;
use reference::bench::{
    BenchReport, INJECTED_FILE, REPORT_FILE, ReportedClass, TARGETS_FILE, read_detections, read_report, read_targets,
};
use reference::location::{committed, recorded_bench};
use reference::recording::Recording;
use types::rate::FrameRate;
use types::size::SourceSize;

const CI_TOLERANCE: f64 = 5e-4;
const COMPLETENESS_TOLERANCE: f64 = 5e-4;
const FRAGMENTS_TOLERANCE: f64 = 5e-3;
const EFFECTIVE_TOLERANCE: f64 = 5e-2;
const EXACT: f64 = 1e-12;

#[cfg(test)]
fn recorded_run(name: &str) -> (Run, BenchReport) {
    let recording = Recording::read(&committed("video_092_original")).unwrap();
    let summary = &recording.summary;
    let folder = recorded_bench(name).unwrap();
    let targets = read_targets(&folder.join(TARGETS_FILE))
        .unwrap()
        .iter()
        .map(|line| recorded::injected(line).unwrap())
        .collect();
    let run = Run {
        video: Video {
            path: summary.video.name.clone().into(),
            size: SourceSize::new(
                NonZeroU32::new(summary.stream.width).unwrap(),
                NonZeroU32::new(summary.stream.height).unwrap(),
            ),
            rate: FrameRate::new(summary.stream.avg_frame_rate).unwrap(),
            damaged: summary.damaged_spans.clone(),
            reported_frames: summary.counts.damaged_frames_reported,
            half_window: NonZeroUsize::new(summary.background_half_window).unwrap(),
        },
        detect: summary.detect_settings::<DetectSettings>().unwrap(),
        reference: recording.frames.detections().clone(),
        injected: read_detections(&folder.join(INJECTED_FILE)).unwrap(),
        targets,
        provenance: Provenance::Replayed(folder.join(TARGETS_FILE)),
    };
    (run, read_report(&folder.join(REPORT_FILE)).unwrap())
}

#[cfg(test)]
fn assert_close(name: &str, field: &str, value: f64, expected: f64, tolerance: f64) {
    assert!(
        (value - expected).abs() <= tolerance,
        "{name} {field}: {value} for {expected}"
    );
}

#[cfg(test)]
fn assert_same_targets(name: &str, evaluation: &Evaluation, report: &BenchReport) {
    assert_eq!(evaluation.targets.len(), report.bats.len(), "{name}");
    for (result, expected) in evaluation.targets.iter().zip(&report.bats) {
        let label = format!("{name} target {}", expected.id);
        let class = result.class;
        assert_eq!(
            (result.id, class.motion().name(), class.amplitude(), class.sigma()),
            (
                expected.id,
                expected.motion.as_str(),
                expected.amplitude,
                expected.sigma
            ),
            "{label}"
        );
        assert_eq!(
            (
                result.visible_frames,
                result.found,
                result.start_delay,
                result.end_early,
                result.fragments
            ),
            (
                expected.visible_frames,
                expected.found,
                expected.start_delay,
                expected.end_early,
                expected.fragments
            ),
            "{label}"
        );
        assert_close(
            &label,
            "completeness",
            result.completeness,
            expected.completeness,
            EXACT,
        );
        assert_close(
            &label,
            "median_effective",
            result.median_effective,
            expected.median_effective,
            EXACT,
        );
    }
}

#[cfg(test)]
fn assert_same_optional(label: &str, field: &str, value: Option<f64>, expected: Option<f64>, tolerance: f64) {
    match (value, expected) {
        (Some(value), Some(expected)) => assert_close(label, field, value, expected, tolerance),
        (value, expected) => assert_eq!(value, expected, "{label} {field}"),
    }
}

#[cfg(test)]
fn assert_same_summary(name: &str, rows: &[ClassSummary], expected: &[ReportedClass]) {
    assert_eq!(rows.len(), expected.len(), "{name}");
    for (row, expected) in rows.iter().zip(expected) {
        let label = format!("{name} {} {} {}", expected.motion, expected.amplitude, expected.sigma);
        let class = row.class;
        assert_eq!(
            (
                class.motion().name(),
                class.amplitude(),
                class.sigma(),
                row.n,
                row.found
            ),
            (
                expected.motion.as_str(),
                expected.amplitude,
                expected.sigma,
                expected.n,
                expected.found
            ),
            "{label}"
        );
        assert_close(
            &label,
            "ci95 low",
            row.found_ci95.0,
            expected.found_ci95[0],
            CI_TOLERANCE,
        );
        assert_close(
            &label,
            "ci95 high",
            row.found_ci95.1,
            expected.found_ci95[1],
            CI_TOLERANCE,
        );
        assert_close(
            &label,
            "median_completeness",
            row.median_completeness,
            expected.median_completeness,
            COMPLETENESS_TOLERANCE,
        );
        assert_same_optional(
            &label,
            "median_start_delay",
            row.median_start_delay,
            expected.median_start_delay,
            0.0,
        );
        assert_same_optional(
            &label,
            "median_end_early",
            row.median_end_early,
            expected.median_end_early,
            0.0,
        );
        assert_same_optional(
            &label,
            "mean_fragments",
            row.mean_fragments,
            expected.mean_fragments,
            FRAGMENTS_TOLERANCE,
        );
        assert_close(
            &label,
            "median_effective",
            row.median_effective,
            expected.median_effective,
            EFFECTIVE_TOLERANCE,
        );
    }
}

#[test]
#[ignore = "local"]
fn the_detections_and_targets_recorded_by_the_v011_give_back_its_bench_report_target_by_target() {
    let benches = [
        ("transit", (17, 0, 14, 163), (253, 148)),
        ("hunt", (136, 0, 14, 515), (584, 456)),
    ];
    for (name, totals, (evaluated, found)) in benches {
        let (run, report) = recorded_run(name);

        let evaluation = evaluate(&run, Judging::default()).unwrap();

        let found_here = evaluation.targets.iter().filter(|result| result.found).count();
        assert_eq!((evaluation.targets.len(), found_here), (evaluated, found), "{name}");

        assert_eq!(
            (
                evaluation.not_visible,
                evaluation.false_tracks,
                evaluation.reference_tracks,
                evaluation.injected_tracks
            ),
            totals,
            "{name}"
        );
        assert_eq!(
            (
                report.not_visible,
                report.false_tracks,
                report.reference_tracks,
                report.injected_tracks
            ),
            totals,
            "{name}"
        );
        assert_same_targets(name, &evaluation, &report);
        assert_same_summary(name, &summarize(&evaluation), &report.summary);
    }
}
