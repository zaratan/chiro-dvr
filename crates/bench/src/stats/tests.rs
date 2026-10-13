use super::{summarize, wilson};
use crate::metrics::{Evaluation, TargetResult};
use crate::synthetic::{Class, Motion};

fn close((low, high): (f64, f64), (expected_low, expected_high): (f64, f64)) -> bool {
    (low - expected_low).abs() < 1e-12 && (high - expected_high).abs() < 1e-12
}

fn result(id: usize, class: Class, found: bool) -> TargetResult {
    TargetResult {
        id,
        class,
        visible_frames: 20,
        found,
        completeness: if found { 1.0 } else { 0.0 },
        start_delay: found.then_some(id),
        end_early: found.then_some(0),
        fragments: usize::from(found) + id % 2,
        median_effective: -40.0 - id as f64,
    }
}

fn evaluation(targets: Vec<TargetResult>) -> Evaluation {
    Evaluation {
        targets,
        not_visible: 0,
        false_tracks: 0,
        reference_tracks: 0,
        injected_tracks: 0,
    }
}

#[test]
fn the_wilson_interval_gives_the_values_of_the_v011() {
    assert_eq!(wilson(0, 0), (0.0, 0.0));
    assert!(close(wilson(0, 28), (0.0, 0.120_647_203_658_107_62)));
    assert!(close(
        wilson(27, 28),
        (0.822_874_238_815_013_5, 0.993_667_643_502_458_2)
    ));
    assert!(close(wilson(28, 28), (0.879_352_796_341_892_3, 1.0)));
    assert!(close(
        wilson(15, 29),
        (0.344_307_379_705_691_6, 0.686_141_806_692_047_8)
    ));
}

#[test]
fn classes_are_summarized_motion_first_then_by_increasing_amplitude_and_sigma() {
    let class = |amplitude, sigma, motion| Class::new(amplitude, sigma, motion).unwrap();
    let hunt = class(-60.0, 3.0, Motion::Hunt);
    let pass = class(-30.0, 3.0, Motion::Pass);
    let darker = class(-60.0, 3.0, Motion::Pass);
    let wider = class(-60.0, 5.0, Motion::Pass);

    let rows = summarize(&evaluation(vec![
        result(0, hunt, false),
        result(1, pass, true),
        result(2, wider, true),
        result(3, darker, true),
        result(4, Class::new(-60.0, 3.0, Motion::Circle).unwrap(), true),
    ]));

    let order: Vec<_> = rows.iter().map(|row| (row.class, row.found)).collect();
    assert_eq!(
        order,
        [
            (darker, 1),
            (wider, 1),
            (pass, 1),
            (hunt, 0),
            (Class::new(-60.0, 3.0, Motion::Circle).unwrap(), 1)
        ]
    );
}

#[test]
fn a_class_summary_takes_medians_over_its_targets_and_delays_and_fragments_over_those_found() {
    let class = Class::new(-15.0, 1.5, Motion::Pass).unwrap();

    let rows = summarize(&evaluation(vec![
        result(1, class, true),
        result(2, class, false),
        result(3, class, true),
        result(5, class, true),
    ]));

    let row = rows[0];
    assert_eq!((row.n, row.found), (4, 3));
    assert!(close(row.found_ci95, wilson(3, 4)));
    assert_eq!(row.median_completeness, 1.0);
    assert_eq!(row.median_start_delay, Some(3.0));
    assert_eq!(row.median_end_early, Some(0.0));
    assert_eq!(row.mean_fragments, Some(2.0));
    assert_eq!(row.median_effective, -42.5);
}

#[test]
fn a_class_where_nothing_is_found_has_no_delay_and_no_fragment_count() {
    let class = Class::new(-15.0, 5.0, Motion::Pass).unwrap();

    let row = summarize(&evaluation(vec![result(0, class, false), result(2, class, false)]))[0];

    assert_eq!(
        (row.median_start_delay, row.median_end_early, row.mean_fragments),
        (None, None, None)
    );
    assert_eq!((row.found, row.median_completeness), (0, 0.0));
}
