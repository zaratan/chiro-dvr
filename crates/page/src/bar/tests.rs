use super::{data, render};
use crate::model::{Page, VerdictsFile};
use crate::testing::Named;

fn page(verdicts: Option<VerdictsFile>) -> Page {
    Page {
        title: "nuit <12>".to_owned(),
        entries: Vec::new(),
        refused: None,
        verdicts,
    }
}

#[test]
fn the_bar_names_the_evening_folder_for_the_advice_after_a_download() {
    let bar = render(&page(None), &Named);
    assert!(bar.contains(r#"data-folder="nuit &lt;12&gt;""#));
    assert!(bar.contains(r#"<input type="file" class="reload-file" accept=".csv,text/csv" hidden>"#));
}

#[test]
fn the_verdicts_file_is_embedded_byte_for_byte_with_its_name_and_how_many_were_found() {
    let file = VerdictsFile {
        name: "verdicts (1).csv".to_owned(),
        modified: None,
        candidates: 3,
        bytes: b"vid\xe9o;piste".to_vec(),
    };
    let embedded = data(&page(Some(file)), &Named);
    assert!(embedded.contains(
        r#"<template id="verdicts-file" data-name="verdicts (1).csv" data-date="" data-candidates="3" data-bytes="dmlk6W87cGlzdGU="></template>"#
    ));
}

#[test]
fn without_a_verdicts_file_nothing_is_embedded() {
    assert!(!data(&page(None), &Named).contains("verdicts-file"));
}

#[test]
fn every_text_the_script_needs_is_given_by_its_key() {
    let embedded = data(&page(None), &Named);
    assert!(embedded.contains(r#""counts":"Script(Counts)""#));
    assert!(embedded.contains(r#""value-doubt":"Script(ValueDoubt)""#));
}
