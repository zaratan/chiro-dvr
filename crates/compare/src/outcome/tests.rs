use reference::location::{Reference, committed};

use super::Outcome;
use crate::case::{Case, EXTRACT, WHOLE};
use crate::comparison::{Comparison, compared};
use crate::error::Error;
use crate::inputs::InputGap;
use crate::sameness::Sameness;
use crate::testing::{extract, frames_without_detections, nothing_detected, pass_finding, same_pass};

fn agreeing() -> Comparison {
    let recording = extract();
    compared(EXTRACT, &same_pass(&recording), &recording, 1.0).unwrap()
}

fn disagreeing() -> Comparison {
    let recording = extract();
    compared(
        EXTRACT,
        &pass_finding(&recording, nothing_detected(&recording)),
        &recording,
        1.0,
    )
    .unwrap()
}

fn failed() -> Error {
    Error::HalfWindow {
        video: "video.mp4".to_owned(),
        expected: 15,
        found: 30,
    }
}

fn cases(count: usize) -> Vec<Case> {
    [EXTRACT, WHOLE]
        .into_iter()
        .take(count)
        .map(|name| Case::committed(Reference::V020, name))
        .collect()
}

#[test]
fn each_answer_is_sorted_into_compared_or_failed_with_its_recording() {
    let outcome = Outcome::of(&cases(2), vec![Ok(agreeing()), Err(failed())]);

    assert_eq!(outcome.compared.len(), 1);
    let [(recording, error)] = outcome.failed.as_slice() else {
        panic!("{:?}", outcome.failed);
    };
    assert_eq!(recording, &committed(Reference::V020, WHOLE));
    assert!(matches!(error, Error::HalfWindow { .. }));
}

#[test]
fn a_failure_other_input_frames_or_no_comparison_at_all_fails_the_run() {
    let with_failure = Outcome::of(&cases(2), vec![Ok(agreeing()), Err(failed())]);
    let other_frames = Outcome::of(
        &cases(1),
        vec![Err(Error::Inputs {
            video: EXTRACT.to_owned(),
            gap: InputGap::FrameCount {
                previous: 1230,
                current: 1229,
            },
        })],
    );
    let only_failed = Outcome::of(&cases(1), vec![Err(failed())]);

    assert!(!with_failure.holds());
    assert!(!other_frames.holds());
    assert!(!only_failed.holds());
    assert!(!Outcome::of(&[], Vec::new()).holds());
    assert!(Outcome::of(&cases(1), vec![Ok(agreeing())]).holds());
}

#[test]
fn a_comparison_that_differs_from_the_reference_is_reported_but_does_not_fail_the_run() {
    let differing = disagreeing();

    assert!(differing.differs());
    assert!(Outcome::of(&cases(1), vec![Ok(differing)]).holds());
}

#[test]
fn a_run_that_differs_from_the_earlier_one_outside_the_damaged_spans_fails_even_in_agreement_with_the_reference() {
    let recording = extract();
    let name = &recording.summary.video.name;
    let same = Comparison {
        same_as: Some(Sameness::of(name, &recording.frames, &recording.frames, &[])),
        ..agreeing()
    };
    let changed = Comparison {
        same_as: Some(Sameness::of(
            name,
            &recording.frames,
            &frames_without_detections(&recording),
            &[],
        )),
        ..agreeing()
    };

    assert!(Outcome::of(&cases(1), vec![Ok(same)]).holds());
    assert!(!Outcome::of(&cases(1), vec![Ok(changed)]).holds());
}
