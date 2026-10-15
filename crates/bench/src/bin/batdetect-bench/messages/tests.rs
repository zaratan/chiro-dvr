use std::path::Path;

use bench::guard::{Change, ClassGuard, Guard};
use bench::metrics::Evaluation;
use bench::stats::ClassSummary;
use bench::synthetic::{self, Class, Motion};
use engine::settings::{Bound, Error, Region, Setting};
use tracking::error::Error as TrackingError;
use tracking::setting::Setting as TrackingSetting;
use types::frame::FrameIndex;

use super::{causes, class_refused, detection_refused, failed, guard, match_refused, table, tracking_refused};

fn evaluation() -> Evaluation {
    Evaluation {
        targets: Vec::new(),
        not_visible: 17,
        false_tracks: 1,
        reference_tracks: 14,
        injected_tracks: 163,
    }
}

fn class() -> Class {
    Class::new(-60.0, 1.5, Motion::Pass).unwrap()
}

#[test]
fn the_table_has_one_row_per_class_with_rounded_values_and_the_totals_last() {
    let rows = [
        ClassSummary {
            class: class(),
            n: 28,
            found: 27,
            found_ci95: (0.822_874_238_8, 0.993_667_643_5),
            median_completeness: 0.894_2,
            median_start_delay: Some(0.0),
            median_end_early: Some(1.5),
            mean_fragments: Some(1.037),
            median_effective: -45.93,
        },
        ClassSummary {
            class: Class::new(-15.0, 5.0, Motion::Hunt).unwrap(),
            n: 29,
            found: 0,
            found_ci95: (0.0, 0.117),
            median_completeness: 0.0,
            median_start_delay: None,
            median_end_early: None,
            mean_fragments: None,
            median_effective: -12.0,
        },
    ];

    let text = table(&rows, &evaluation());

    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 4);
    assert!(
        lines[0].starts_with("motion  amplitude  sigma   n  found  ci95"),
        "{text}"
    );
    assert_eq!(
        lines[1],
        "pass        -60.0    1.5  28     27  [0.823, 0.994]         0.894          0.0        1.5       1.04      -45.9"
    );
    assert!(
        lines[2].starts_with("hunt        -15.0    5.0  29      0  [0.000, 0.117]"),
        "{text}"
    );
    assert!(lines[2].contains("  -  "), "{text}");
    assert_eq!(
        lines[3],
        "not visible: 17  false tracks: 1  tracks: 163 injected run, 14 reference"
    );
}

#[test]
fn the_guard_text_lists_each_class_its_lost_and_gained_targets_and_those_seen_on_one_side() {
    let lost = Change {
        id: 12,
        start: FrameIndex::new(4567),
    };
    let checked = Guard {
        classes: vec![ClassGuard {
            class: class(),
            n: 28,
            v011_found: 27,
            found: 26,
            lowest: 0.823,
            lost: vec![lost],
            gained: Vec::new(),
        }],
        v011_only: vec![Change {
            id: 3,
            start: FrameIndex::new(10),
        }],
        rust_only: Vec::new(),
        contrast_differs: Vec::new(),
        v011_false_tracks: 0,
        false_tracks: 1,
        v011_reference_tracks: 14,
        reference_tracks: 14,
    };

    let text = guard(Path::new("transit"), &checked);

    assert_eq!(
        text,
        "guard against transit: passed\n\
         visibility decided by the contrast the v0.1.1 recorded; the table above and bench.json use the contrast the Rust \
         measured\n\
         pass -60 σ 1.5: 28 visible on both sides, v0.1.1 found 27, Rust found 26, lower bound 0.823: passes\n  \
         lost: 12 (frame 4567)\n  gained: -\n\
         visible for the v0.1.1 only (1): 3 (frame 10)\nvisible for the Rust only (0): -\n\
         visibility changed by the contrast the Rust measured (0): -\nfalse tracks: v0.1.1 0, Rust 1\n\
         reference tracks: v0.1.1 14, Rust 14"
    );
}

#[test]
fn a_class_below_its_bound_makes_the_guard_fail() {
    let checked = Guard {
        classes: vec![ClassGuard {
            class: class(),
            n: 30,
            v011_found: 30,
            found: 20,
            lowest: 0.886,
            lost: Vec::new(),
            gained: Vec::new(),
        }],
        v011_only: Vec::new(),
        rust_only: Vec::new(),
        contrast_differs: Vec::new(),
        v011_false_tracks: 0,
        false_tracks: 0,
        v011_reference_tracks: 14,
        reference_tracks: 14,
    };

    let text = guard(Path::new("hunt"), &checked);

    assert!(text.starts_with("guard against hunt: FAILED"), "{text}");
    assert!(text.contains("lower bound 0.886: below the bound"), "{text}");
}

#[test]
fn every_detection_setting_is_named_by_its_option() {
    for (setting, option) in [
        (Setting::Threshold, "--threshold"),
        (Setting::MinArea, "--min-area"),
        (Setting::MaxArea, "--max-area"),
        (Setting::BgWindowS, "--bg-window"),
        (Setting::BgStep, "--bg-step"),
        (Setting::MergeRadius, "--merge-radius"),
        (Setting::WorkWidth, "--work-width"),
        (Setting::NoiseFactor, "--noise-factor"),
        (Setting::TargetSigma, "--target-sigma"),
    ] {
        let refused = Error::Setting {
            setting,
            value: -1.0,
            bound: Bound::Positive,
        };

        assert_eq!(
            detection_refused(&refused),
            format!("{option} must be a finite number > 0, not -1")
        );
    }
    let region = Error::Region {
        index: 1,
        region: Region {
            x0: 0.5,
            y0: 0.0,
            x1: 0.2,
            y1: 1.0,
        },
    };
    assert_eq!(
        detection_refused(&region),
        "--osd-region 2 does not satisfy 0 ≤ x0 < x1 ≤ 1 and 0 ≤ y0 < y1 ≤ 1: 0.5,0,0.2,1"
    );
}

#[test]
fn every_tracking_setting_is_named_by_its_option() {
    for (setting, option) in [
        (TrackingSetting::PadS, "--unstable-pad"),
        (TrackingSetting::MaxJump, "--max-jump"),
        (TrackingSetting::MinTravel, "--min-travel"),
        (TrackingSetting::TwinDistance, "--twin-distance"),
        (TrackingSetting::MaxMedianTurn, "--max-median-turn"),
    ] {
        let refused = TrackingError::Setting { setting, value: -2.0 };

        assert_eq!(tracking_refused(&refused), format!("{option} is out of bounds: -2"));
    }
}

#[test]
fn errors_without_an_option_keep_their_own_message() {
    let purity = bench::config::Error::Purity(2.0);
    let empty = synthetic::Error::Empty { id: 3 };

    assert_eq!(match_refused(&purity), purity.to_string());
    assert_eq!(class_refused(&empty), empty.to_string());
}

#[test]
fn a_failure_names_the_video_then_every_cause() {
    let refused = bench::recorded::Error::Motion {
        id: 4,
        source: synthetic::UnknownMotion("glide".to_owned()),
    };

    assert_eq!(
        failed(Path::new("video.mp4"), &refused),
        "video.mp4: recorded target 4 has an unknown motion: unknown motion \"glide\", expected pass, hunt or circle"
    );
    assert_eq!(
        causes(&refused),
        failed(Path::new(""), &refused).trim_start_matches(": ")
    );
}
