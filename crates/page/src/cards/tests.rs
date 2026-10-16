use super::{failed, pending};
use crate::labels::Label;
use crate::model::{Date, Failed};
use crate::testing::{Named, video};

fn damaged() -> Failed {
    Failed {
        name: "video_093".to_owned(),
        date: Some(Date {
            year: 2026,
            month: 10,
            day: 16,
            hour: 10,
            minute: 19,
        }),
        cause: "damaged".to_owned(),
        place: "/nuit/batdetect/video_093".to_owned(),
        folder: "/nuit/batdetect/video_093".to_owned(),
        detail: Some("cannot decode".to_owned()),
        previous: None,
    }
}

#[test]
fn a_failure_shows_the_reason_and_the_advice_of_the_summary_with_its_detail() {
    let card = failed(&damaged(), "v3", &Named);
    assert!(card.contains(r#"<section class="video failed" id="v3""#));
    assert!(card.contains("Failure { video: &quot;video_093&quot;, cause: &quot;damaged&quot;"));
    assert!(card.contains("Todo { cause: &quot;damaged&quot;"));
    assert!(card.contains("Detail(&quot;cannot decode&quot;)"));
    assert!(card.contains("Tried(Date { year: 2026"));
    assert!(!card.contains("class=\"previous\""));
}

#[test]
fn a_failure_after_a_success_links_to_the_results_of_the_earlier_run() {
    let mut night = damaged();
    night.previous = Some(video("video_093"));
    let card = failed(&night, "v3", &Named);
    assert!(card.contains(r##"<a class="previous" href="#v3-previous">"##));
    assert!(card.contains(r#"<details class="video" id="v3-previous">"#));
}

#[test]
fn a_video_still_waiting_or_left_unfinished_is_named_with_its_state() {
    let card = pending("video_094", "v4", Label::WaitingBadge, Label::WaitingText, &Named);
    assert!(card.contains(">video_094</h2>"));
    assert!(card.contains("WaitingBadge"));
    assert!(card.contains("WaitingText"));
}
