use super::{GENERATOR, is_ours, render};
use crate::model::{Entry, Failed, Page, Refusals, RefusedInput, TooMany};
use crate::testing::{Named, video};

fn evening() -> Page {
    let mut crowded = video("video_097");
    crowded.too_many = Some(TooMany { count: 301, limit: 300 });
    Page {
        title: "nuit-12-oct".to_owned(),
        refused: None,
        entries: vec![
            Entry::Video(video("video_092")),
            Entry::Video(crowded),
            Entry::Failed(Failed {
                name: "video_093".to_owned(),
                date: None,
                cause: "damaged".to_owned(),
                place: String::new(),
                folder: String::new(),
                detail: None,
                previous: None,
            }),
            Entry::Unfinished {
                name: "video_094".to_owned(),
            },
            Entry::Older {
                name: "video_096".to_owned(),
            },
            Entry::Waiting {
                name: "video_095".to_owned(),
            },
        ],
    }
}

#[test]
fn the_page_carries_the_mark_that_lets_a_later_run_replace_it() {
    let html = render(&evening(), &Named);
    assert!(html.starts_with("<!doctype html>\n<html lang=\"fr\">"));
    assert!(is_ours(html.as_bytes()));
}

#[test]
fn a_page_written_by_someone_else_is_not_ours() {
    assert!(!is_ours(b"<!doctype html><title>Mes notes</title>"));
    let far = format!("{}{GENERATOR}", " ".repeat(4096));
    assert!(!is_ours(far.as_bytes()));
}

#[test]
fn the_total_counts_each_state_and_every_track_and_links_to_the_first_video_of_each_problem() {
    let html = render(&evening(), &Named);
    for count in [
        "Videos(6)",
        "Done(1)",
        "UnfinishedCount(1)",
        "WaitingCount(1)",
        "Tracks(8)",
    ] {
        assert!(html.contains(count), "{count}");
    }
    assert!(html.contains(r##"<a class="count bad" href="#v3">FailedCount(1)</a>"##));
    assert!(html.contains(r##"<a class="count bad" href="#v2">TooManyCount(1)</a>"##));
}

#[test]
fn the_entries_follow_the_order_given_with_their_anchors() {
    let html = render(&evening(), &Named);
    let anchors: Vec<usize> = [
        "id=\"v1\"",
        "id=\"v2\"",
        "id=\"v3\"",
        "id=\"v4\"",
        "id=\"v5\"",
        "id=\"v6\"",
    ]
    .iter()
    .map(|anchor| html.find(anchor).unwrap())
    .collect();
    assert!(anchors.is_sorted());
}

#[test]
fn a_video_done_by_an_older_version_says_so_rather_than_unfinished() {
    let html = render(&evening(), &Named);
    let (_, card) = html.split_once(">video_096</h2>").unwrap();
    assert!(card.starts_with("\n    <span class=\"badge\">OlderBadge</span>"));
    assert!(card.contains("OlderText"));
}

#[test]
fn a_page_with_nothing_to_fold_has_no_button_to_unfold_everything() {
    let page = Page {
        title: "nuit".to_owned(),
        refused: None,
        entries: vec![Entry::Waiting {
            name: "video_095".to_owned(),
        }],
    };
    assert!(!render(&page, &Named).contains(r#"class="toggle-all""#));
}

#[test]
fn nothing_is_loaded_from_the_network_and_videos_wait_for_a_click() {
    let html = render(&evening(), &Named);
    assert!(!html.contains("http://") && !html.contains("https://"));
    assert!(!html.contains("fetch("));
    assert_eq!(
        html.matches("<video ").count(),
        html.matches(r#"preload="none""#).count()
    );
}

#[test]
fn the_title_of_the_folder_is_escaped() {
    let page = Page {
        title: "<nuit>".to_owned(),
        refused: None,
        entries: Vec::new(),
    };
    let html = render(&page, &Named);
    assert!(html.contains("<title>Heading — &lt;nuit&gt;</title>"));
}

fn refused_evening() -> Page {
    Page {
        refused: Some(Refusals {
            date: None,
            inputs: vec![RefusedInput {
                path: "/nuit/video_093.mp4".to_owned(),
                reason: "refused".to_owned(),
                cause: None,
                todo: "read-refused".to_owned(),
                videos: String::new(),
                detail: None,
            }],
        }),
        ..evening()
    }
}

#[test]
fn the_refused_inputs_come_before_the_videos() {
    let html = render(&refused_evening(), &Named);
    let card = html
        .find(r#"<section class="video failed refused" id="refused""#)
        .unwrap();
    let first_video = html.find(r#"id="v1""#).unwrap();
    assert!(card < first_video);
}

#[test]
fn the_total_links_to_the_refused_inputs() {
    let html = render(&refused_evening(), &Named);
    assert!(html.contains(r##"<a class="count bad" href="#refused">RefusedCount(1)</a>"##));
}

#[test]
fn a_launch_without_refusal_shows_neither_the_card_nor_its_count() {
    let html = render(&evening(), &Named);
    assert!(!html.contains(r#"id="refused""#));
    assert!(!html.contains("RefusedCount"));
}
