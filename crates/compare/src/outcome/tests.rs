use reference::location::committed;
use reference::recording::Recording;

use super::Outcome;
use crate::case::{Case, EXTRACT, WHOLE};
use crate::comparison::{Comparison, compared};
use crate::error::Error;
use crate::testing::nothing_detected;

fn agreeing() -> Comparison {
    let recording = Recording::read(&committed(EXTRACT)).unwrap();
    compared(EXTRACT, recording.frames.detections(), &recording, 1.0).unwrap()
}

fn disagreeing() -> Comparison {
    let recording = Recording::read(&committed(EXTRACT)).unwrap();
    let empty = nothing_detected(&recording);
    compared(EXTRACT, &empty, &recording, 1.0).unwrap()
}

fn failed() -> Error {
    Error::HalfWindow {
        video: "video.mp4".to_owned(),
        expected: 15,
        found: 30,
    }
}

fn cases(count: usize) -> Vec<Case> {
    [EXTRACT, WHOLE].into_iter().take(count).map(Case::committed).collect()
}

#[test]
fn each_answer_is_sorted_into_compared_or_failed_with_its_recording() {
    let outcome = Outcome::of(&cases(2), vec![Ok(agreeing()), Err(failed())]);

    assert_eq!(outcome.compared.len(), 1);
    let [(recording, error)] = outcome.failed.as_slice() else {
        panic!("{:?}", outcome.failed);
    };
    assert_eq!(recording, &committed(WHOLE));
    assert!(matches!(error, Error::HalfWindow { .. }));
}

#[test]
fn a_failure_a_comparison_under_98_percent_or_no_comparison_at_all_fails_the_run() {
    let with_failure = Outcome::of(&cases(2), vec![Ok(agreeing()), Err(failed())]);
    let under = Outcome::of(&cases(1), vec![Ok(disagreeing())]);
    let only_failed = Outcome::of(&cases(1), vec![Err(failed())]);

    assert!(!with_failure.holds());
    assert!(!under.holds());
    assert!(!only_failed.holds());
    assert!(Outcome::of(&cases(1), vec![Ok(agreeing())]).holds());
}
