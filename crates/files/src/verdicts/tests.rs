use std::fs;
use std::path::{Path, PathBuf};

use types::language::Language;

use super::read_verdicts;
use crate::params::TrackEntry;
use crate::table::{Column, Headings};

#[derive(Debug)]
struct Named;

impl Headings for Named {
    fn heading(&self, column: Column, language: Language) -> String {
        format!("{column:?}{}", if language == Language::French { "_fr" } else { "" })
    }
}

fn track(id: usize, start_s: f64) -> TrackEntry {
    TrackEntry {
        id,
        start_s,
        duration_s: 0.6,
        color: None,
        summary: None,
        clip: None,
        zoom: None,
    }
}

fn written(folder: &Path, text: &str) -> PathBuf {
    let path = folder.join("v_pistes.csv");
    fs::write(&path, text).unwrap();
    path
}

#[test]
fn a_verdict_noted_in_a_french_table_is_read_back_for_its_track() {
    let folder = tempfile::tempdir().unwrap();
    let path = written(
        folder.path(),
        "\u{feff}Track_fr;Verdict_fr;StartSeconds_fr\r\n1;chauve-souris;4,8\r\n2;;23,48\r\n",
    );
    let tracks = [track(1, 4.801), track(2, 23.48)];
    assert_eq!(
        read_verdicts(&path, &tracks, &Named),
        [Some("chauve-souris".to_owned()), None]
    );
}

#[test]
fn a_table_saved_again_by_a_spreadsheet_with_quotes_keeps_its_verdicts() {
    let folder = tempfile::tempdir().unwrap();
    let path = written(
        folder.path(),
        "Track,Start,Verdict,StartSeconds\n1,0:04.80,\"oui, pipistrelle; \"\"sûre\"\"\",4.80\n",
    );
    assert_eq!(
        read_verdicts(&path, &[track(1, 4.8)], &Named),
        [Some("oui, pipistrelle; \"sûre\"".to_owned())]
    );
}

#[test]
fn a_table_kept_from_another_run_gives_no_verdict_to_tracks_that_moved() {
    let folder = tempfile::tempdir().unwrap();
    let path = written(folder.path(), "Track;Verdict;StartSeconds\n1;oui;4,8\n");
    assert_eq!(read_verdicts(&path, &[track(1, 6.2)], &Named), [None]);
}

#[test]
fn a_table_without_a_verdict_column_or_without_a_file_gives_no_verdict() {
    let folder = tempfile::tempdir().unwrap();
    let legacy = written(folder.path(), "id,start,end,start_s\n1,0:04.80,0:05.40,4.8\n");
    assert_eq!(read_verdicts(&legacy, &[track(1, 4.8)], &Named), [None]);
    let absent = folder.path().join("absent.csv");
    assert_eq!(read_verdicts(&absent, &[track(1, 4.8)], &Named), [None]);
}
