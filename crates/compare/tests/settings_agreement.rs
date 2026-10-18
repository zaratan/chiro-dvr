use std::io;
use std::sync::OnceLock;

use compare::case::Target;
use compare::comparison::{Folders, Gaps, VIDEOS_AT_ONCE, compare_all};
use compare::outcome::Outcome;
use compare::report::write_report;
use reference::location::Reference;

#[cfg(test)]
fn outcome() -> &'static Outcome {
    static OUTCOME: OnceLock<Outcome> = OnceLock::new();
    OUTCOME.get_or_init(|| {
        let cases = Target::Settings.cases(Reference::V020).unwrap();
        let outcome = Outcome::of(&cases, compare_all(&cases, VIDEOS_AT_ONCE, &Folders::default()));
        write_report(&outcome.compared, &mut io::stdout().lock()).unwrap();
        outcome
    })
}

#[test]
#[ignore = "settings"]
fn every_setting_of_the_bench_is_compared_and_none_fails() {
    let outcome = outcome();
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
fn every_setting_keeps_the_gaps_frozen_against_the_v020_case_by_case_where_only_weak_detections_are_added() {
    let found: Vec<(&str, Gaps)> = outcome()
        .compared
        .iter()
        .map(|comparison| (comparison.detections.name(), comparison.gaps()))
        .collect();
    let weak = Gaps::added_only;
    let agreeing = Gaps::agreeing;

    assert_eq!(
        found,
        [
            ("max_area_400", weak(6, 6, [4, 4, 0, 1])),
            ("merge_radius_0", weak(12, 16, [4, 4, 0, 0])),
            ("min_area_9", weak(2, 2, [5, 4, 0, 0])),
            ("noise_factor_0", weak(9, 10, [4, 4, 0, 0])),
            ("osd", weak(5, 5, [4, 4, 0, 0])),
            ("quick", weak(4, 6, [4, 4, 0, 0])),
            ("sigma_0", weak(264, 664, [24, 22, 6, 16])),
            ("sigma_3", weak(5, 5, [3, 3, 1, 1])),
            ("size_720x540", weak(3, 3, [4, 4, 0, 0])),
            ("step_1", weak(6, 6, [5, 5, 0, 0])),
            ("threshold_25", agreeing(4)),
            ("width_1440", weak(1, 1, [4, 4, 0, 0])),
            ("width_1440_osd", weak(1, 1, [4, 4, 0, 0])),
            ("width_480", weak(8, 8, [4, 4, 0, 0])),
            ("width_720", weak(13, 16, [5, 4, 0, 2])),
            ("window_2s", weak(6, 6, [4, 4, 0, 0])),
        ]
    );
}
