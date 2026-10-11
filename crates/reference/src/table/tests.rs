use std::fs;

use super::{TrackRow, decimal, read, write};
use crate::error::Error;

const TABLE: &str = "id,start,end,start_s,duration_s,hits,chord_px,path_px,speed_px_s,max_area_px,peak_amplitude,filled_frames\r\n\
                     1,0:07.39,0:08.13,7.39,0.73,20,584.6,660.4,901.4,274,47.0,3\r\n";

fn table(text: &str) -> Result<Vec<TrackRow>, Error> {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tracks.csv");
    fs::write(&path, text).unwrap();
    read(&path)
}

#[test]
fn a_row_reads_every_column() {
    let rows = table(TABLE).unwrap();
    assert_eq!(
        rows,
        [TrackRow {
            id: 1,
            start: "0:07.39".to_owned(),
            end: "0:08.13".to_owned(),
            start_s: 7.39,
            duration_s: 0.73,
            hits: 20,
            chord_px: 584.6,
            path_px: 660.4,
            speed_px_s: 901.4,
            max_area_px: 274,
            peak_amplitude: 47.0,
            filled_frames: 3,
        }]
    );
}

#[test]
fn a_table_is_written_back_byte_for_byte() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("tracks.csv");
    write(&path, &table(TABLE).unwrap()).unwrap();
    assert_eq!(fs::read_to_string(&path).unwrap(), TABLE);
}

#[test]
fn a_table_without_tracks_is_its_header_alone() {
    let header = TABLE.lines().next().unwrap();
    assert_eq!(table(&format!("{header}\r\n")).unwrap(), []);
}

#[test]
fn another_header_is_refused() {
    let error = table("id,start\r\n").unwrap_err();
    assert!(matches!(error, Error::CsvHeader { .. }));
}

#[test]
fn a_row_with_a_missing_column_is_refused_with_its_line() {
    let short = TABLE.replace(",3\r\n", "\r\n");
    let error = table(&short).unwrap_err();
    assert!(matches!(
        error,
        Error::CsvWidth {
            line: 2,
            found: 11,
            expected: 12,
            ..
        }
    ));
}

#[test]
fn a_fractional_count_is_refused_by_column() {
    let error = table(&TABLE.replace(",274,", ",274.5,")).unwrap_err();
    assert!(
        error
            .to_string()
            .ends_with("tracks.csv:2: column max_area_px is not an integer")
    );
}

#[test]
fn a_word_in_a_measure_is_refused_by_column() {
    let error = table(&TABLE.replace(",584.6,", ",far,")).unwrap_err();
    assert!(
        error
            .to_string()
            .ends_with("tracks.csv:2: column chord_px is not a number")
    );
}

#[test]
fn a_missing_table_or_folder_is_reported() {
    let directory = tempfile::tempdir().unwrap();
    let absent = directory.path().join("absent").join("tracks.csv");
    assert!(matches!(read(&absent).unwrap_err(), Error::Read { .. }));
    assert!(matches!(write(&absent, &[]).unwrap_err(), Error::Write { .. }));
}

#[test]
fn a_measure_is_written_in_its_shortest_decimal_form_with_a_point() {
    assert_eq!(decimal(47.0), "47.0");
    assert_eq!(decimal(901.4), "901.4");
    assert_eq!(decimal(0.000_01), "0.00001");
    assert_eq!(decimal(f64::INFINITY), "inf");
}
