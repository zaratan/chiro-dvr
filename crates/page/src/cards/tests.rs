use super::{failed, pending, refused};
use crate::labels::Label;
use crate::model::{Date, Failed, Refusals, RefusedInput};
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

fn unopened() -> RefusedInput {
    RefusedInput {
        path: "/nuit/video_093.mp4".to_owned(),
        reason: "unopened".to_owned(),
        cause: Some("unsupported".to_owned()),
        todo: "unsupported".to_owned(),
        videos: String::new(),
        detail: Some("codec hevc".to_owned()),
    }
}

#[test]
fn each_refused_input_says_its_reason_then_what_to_do() {
    let card = refused(
        &Refusals {
            date: None,
            inputs: vec![unopened()],
        },
        &Named,
    );
    let reason = card.find("RefusedInput { path: &quot;/<wbr>nuit/<wbr>video_093.mp4&quot;, reason: &quot;unopened&quot;, cause: Some(&quot;unsupported&quot;)").unwrap();
    let todo = card
        .find("Todo { cause: &quot;unsupported&quot;, folder: &quot;/nuit/video_093.mp4&quot; }")
        .unwrap();
    assert!(reason < todo);
    assert!(card.contains("Detail(&quot;codec hevc&quot;)"));
}

#[test]
fn the_refused_card_counts_its_inputs_and_dates_the_launch() {
    let date = Date {
        year: 2026,
        month: 10,
        day: 16,
        hour: 21,
        minute: 2,
    };
    let card = refused(
        &Refusals {
            date: Some(date),
            inputs: vec![unopened(), unopened()],
        },
        &Named,
    );
    assert!(card.contains("RefusedHeading(2)"));
    assert!(card.contains("RefusedConsequence(2)"));
    assert!(card.contains("RefusedWhen(Date { year: 2026"));
    assert_eq!(card.matches("<li>").count(), 2);
}

#[test]
fn a_refused_input_without_detail_shows_no_detail_line() {
    let missing = RefusedInput {
        detail: None,
        ..unopened()
    };
    let card = refused(
        &Refusals {
            date: None,
            inputs: vec![missing],
        },
        &Named,
    );
    assert!(!card.contains("class=\"detail\""));
}

#[test]
fn a_long_path_may_break_after_each_folder_rather_than_inside_a_name() {
    let card = refused(
        &Refusals {
            date: None,
            inputs: vec![unopened()],
        },
        &Named,
    );
    assert!(card.contains("path: &quot;/<wbr>nuit/<wbr>video_093.mp4&quot;"));
}

#[test]
fn a_path_with_markup_characters_is_shown_as_text() {
    let odd = RefusedInput {
        path: "/nuit/<b>&.mp4".to_owned(),
        ..unopened()
    };
    let card = refused(
        &Refusals {
            date: None,
            inputs: vec![odd],
        },
        &Named,
    );
    assert!(card.contains("&lt;b&gt;&amp;.mp4"));
    assert!(!card.contains("<b>"));
}

#[test]
fn a_failed_video_may_break_its_paths_after_each_folder() {
    let card = failed(&damaged(), "v3", &Named);
    assert!(card.contains("place: &quot;/<wbr>nuit/<wbr>batdetect/<wbr>video_093&quot;"));
    assert!(card.contains("folder: &quot;/<wbr>nuit/<wbr>batdetect/<wbr>video_093&quot;"));
}
