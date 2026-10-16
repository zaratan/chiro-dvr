use types::language::Language;

use super::carries_verdicts;
use crate::table::{Column, Headings};

const FRENCH_HEADER: &str = "\u{feff}piste;début;fin;durée (s);verdict;points";
const ENGLISH_HEADER: &str = "track,start,end,duration (s),verdict,points";

#[derive(Debug)]
struct Spelled;

impl Headings for Spelled {
    fn heading(&self, column: Column, language: Language) -> String {
        match (column, language) {
            (Column::Verdict, _) => "verdict".to_owned(),
            (_, Language::French) => "autre".to_owned(),
            (_, Language::English) => "other".to_owned(),
        }
    }
}

fn carries(text: &str) -> bool {
    carries_verdicts(text.as_bytes(), &Spelled)
}

#[test]
fn a_verdict_written_in_one_row_is_seen_in_either_language() {
    assert!(carries(&format!(
        "{FRENCH_HEADER}\r\n1;0:00:04;0:00:14;9,62;;290\r\n2;0:00:23;0:00:24;0,57;oui;17\r\n"
    )));
    assert!(carries(&format!(
        "{ENGLISH_HEADER}\r\n1,0:00:04,0:00:14,9.62,yes,290\r\n"
    )));
}

#[test]
fn a_table_whose_verdicts_are_all_empty_carries_none() {
    assert!(!carries(&format!("{FRENCH_HEADER}\r\n1;0:00:04;0:00:14;9,62;;290\r\n")));
    assert!(!carries(&format!("{ENGLISH_HEADER}\r\n")));
}

#[test]
fn a_table_resaved_with_another_separator_or_without_its_byte_order_mark_still_shows_its_verdict() {
    for separator in ["\t", ",", ";"] {
        let header = ["piste", "début", "verdict"].join(separator);
        let row = ["1", "0:00:04", "oui"].join(separator);
        assert!(carries(&format!("{header}\n{row}\n")), "{separator:?}");
    }
}

#[test]
fn a_table_resaved_in_mac_roman_with_old_mac_line_ends_still_shows_its_verdict() {
    let mut bytes = b"piste;d".to_vec();
    bytes.push(0x8E);
    bytes.extend_from_slice(b"but;verdict\r1;0:00:04;chauve-souris\r");
    assert!(carries_verdicts(&bytes, &Spelled));
}

#[test]
fn a_verdict_made_of_spaces_counts_but_a_quoted_empty_cell_does_not() {
    assert!(carries(&format!(
        "{ENGLISH_HEADER}\r\n1,0:00:04,0:00:14,9.62, ,290\r\n"
    )));
    assert!(!carries(&format!(
        "{ENGLISH_HEADER}\r\n1,0:00:04,0:00:14,9.62,\"\",290\r\n"
    )));
}

#[test]
fn a_verdict_column_moved_by_an_inserted_column_quoted_or_capitalised_is_still_found() {
    assert!(carries("piste;notes;\"verdict\"\r\n1;vu deux fois;oui\r\n"));
    assert!(carries("piste;Verdict\r\n1;oui\r\n"));
}

#[test]
fn a_table_without_a_verdict_column_or_without_anything_carries_none() {
    assert!(!carries("id,start,end\r\n1,0:04.80,oui\r\n"));
    assert!(!carries(""));
}
