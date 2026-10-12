use engine::settings::{self, Unsupported};
use reference::location::committed;
use reference::recording::Recording;
use types::detection::FrameDetections;

use super::{Outcome, Refusal};
use crate::case::{Case, EXTRACT, WHOLE};
use crate::comparison::{Comparison, compared};
use crate::error::Error;

fn agreeing() -> Comparison {
    let recording = Recording::read(&committed(EXTRACT)).unwrap();
    compared(EXTRACT, recording.frames.detections(), &recording, 1.0).unwrap()
}

fn disagreeing() -> Comparison {
    let recording = Recording::read(&committed(EXTRACT)).unwrap();
    let empty = FrameDetections::new(vec![Vec::new(); recording.frames.detections().frame_count()]).unwrap();
    compared(EXTRACT, &empty, &recording, 1.0).unwrap()
}

fn refused() -> Error {
    Error::Settings {
        video: "video.mp4".to_owned(),
        source: settings::Error::Unsupported(Unsupported::MergeRadius { value: 0.0 }),
    }
}

fn failed() -> Error {
    Error::HalfWindow {
        video: "video.mp4".to_owned(),
        expected: 15,
        found: 30,
    }
}

fn cases(count: usize) -> Vec<Case> {
    [EXTRACT, WHOLE, "settings/osd"]
        .into_iter()
        .take(count)
        .map(Case::committed)
        .collect()
}

#[test]
fn each_answer_is_sorted_into_compared_refused_or_failed_with_its_recording() {
    let outcome = Outcome::of(&cases(3), vec![Ok(agreeing()), Err(refused()), Err(failed())]);

    assert_eq!(outcome.compared.len(), 1);
    assert_eq!(
        outcome.refused,
        [Refusal {
            recording: committed(WHOLE),
            reason: "merge_radius 0: only 6 is handled".to_owned()
        }]
    );
    let [(recording, error)] = outcome.failed.as_slice() else {
        panic!("{:?}", outcome.failed);
    };
    assert_eq!(recording, &committed("settings/osd"));
    assert!(matches!(error, Error::HalfWindow { .. }));
}

#[test]
fn a_refusal_fails_the_run_unless_the_target_expects_some() {
    let outcome = Outcome::of(&cases(2), vec![Ok(agreeing()), Err(refused())]);

    assert!(outcome.holds(true));
    assert!(!outcome.holds(false));
}

#[test]
fn a_failure_a_comparison_under_98_percent_or_no_comparison_at_all_fails_the_run() {
    let with_failure = Outcome::of(&cases(2), vec![Ok(agreeing()), Err(failed())]);
    let under = Outcome::of(&cases(1), vec![Ok(disagreeing())]);
    let only_refused = Outcome::of(&cases(1), vec![Err(refused())]);

    assert!(!with_failure.holds(true));
    assert!(!under.holds(true));
    assert!(!only_refused.holds(true));
    assert!(Outcome::of(&cases(1), vec![Ok(agreeing())]).holds(false));
}
