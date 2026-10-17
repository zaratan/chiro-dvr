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
fn every_setting_keeps_the_gaps_frozen_against_the_v020_case_by_case() {
    let found: Vec<(&str, Gaps)> = outcome()
        .compared
        .iter()
        .map(|comparison| (comparison.detections.name(), comparison.gaps()))
        .collect();
    let agreeing = Gaps::agreeing;

    assert_eq!(
        found,
        [
            ("max_area_400", agreeing(4)),
            ("merge_radius_0", agreeing(4)),
            ("min_area_9", agreeing(5)),
            ("noise_factor_0", agreeing(4)),
            ("osd", agreeing(4)),
            ("quick", agreeing(4)),
            ("sigma_0", agreeing(30)),
            ("sigma_3", agreeing(4)),
            ("size_720x540", agreeing(4)),
            ("step_1", agreeing(5)),
            ("threshold_25", agreeing(4)),
            ("width_1440", agreeing(4)),
            ("width_1440_osd", agreeing(4)),
            ("width_480", agreeing(4)),
            ("width_720", agreeing(5)),
            ("window_2s", agreeing(4)),
        ]
    );
}
