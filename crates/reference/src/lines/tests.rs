use std::fs;
use std::io::Read;

use flate2::read::MultiGzDecoder;
use serde::{Deserialize, Serialize};

use super::{read, write};
use crate::error::Error;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Line {
    frame: usize,
    amplitude: f64,
}

fn lines() -> Vec<Line> {
    vec![
        Line {
            frame: 0,
            amplitude: 13.370_320_638_020_829,
        },
        Line {
            frame: 1,
            amplitude: 4.0,
        },
    ]
}

#[test]
fn plain_lines_are_one_compact_object_per_line() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("frames.jsonl");
    write(&path, lines()).unwrap();
    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        "{\"frame\":0,\"amplitude\":13.370320638020829}\n{\"frame\":1,\"amplitude\":4.0}\n"
    );
    assert_eq!(read::<Line>(&path).unwrap(), lines());
}

#[test]
fn a_gz_name_is_compressed_and_read_back() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("frames.jsonl.gz");
    write(&path, lines()).unwrap();
    let mut text = String::new();
    MultiGzDecoder::new(fs::File::open(&path).unwrap())
        .read_to_string(&mut text)
        .unwrap();
    assert!(text.starts_with("{\"frame\":0,"));
    assert_eq!(read::<Line>(&path).unwrap(), lines());
}

#[test]
fn the_gzip_header_carries_neither_name_nor_date() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("frames.jsonl.gz");
    write(&path, lines()).unwrap();
    let bytes = fs::read(&path).unwrap();
    assert_eq!(bytes[3], 0);
    assert_eq!(bytes[4..8], [0, 0, 0, 0]);
}

#[test]
fn a_malformed_line_is_reported_with_its_number() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("frames.jsonl");
    fs::write(&path, "{\"frame\":0,\"amplitude\":1.0}\n{\"frame\":1}\n").unwrap();
    let error = read::<Line>(&path).unwrap_err();
    assert!(matches!(error, Error::Json { line: 2, .. }));
    assert!(
        error
            .to_string()
            .ends_with("frames.jsonl:2 does not follow the reference format")
    );
}

#[test]
fn a_missing_file_is_reported_by_path() {
    let error = read::<Line>(std::path::Path::new("absent.jsonl")).unwrap_err();
    assert!(matches!(error, Error::Read { .. }));
    assert!(error.to_string().contains("absent.jsonl"));
}

#[test]
fn a_corrupt_gzip_stream_is_a_read_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("frames.jsonl.gz");
    fs::write(&path, b"not gzip at all").unwrap();
    assert!(matches!(read::<Line>(&path).unwrap_err(), Error::Read { .. }));
}

#[test]
fn a_folder_that_does_not_exist_cannot_be_written() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("absent").join("frames.jsonl");
    assert!(matches!(write(&path, lines()).unwrap_err(), Error::Write { .. }));
}

#[test]
fn a_value_json_cannot_hold_is_an_encoding_error() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("frames.jsonl");
    let error = write(&path, [std::collections::BTreeMap::from([((1, 2), 3)])]).unwrap_err();
    assert!(matches!(error, Error::Encode { .. }));
}
