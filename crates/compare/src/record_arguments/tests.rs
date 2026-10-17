use std::path::PathBuf;

use clap::error::ErrorKind;

use super::{RecordRequest, parse};
use crate::case::Target;

#[test]
fn the_videos_and_the_folder_to_record_into_are_read_with_an_optional_earlier_folder() {
    assert_eq!(
        parse(["record", "local", "--out", "v020", "--same-as", "earlier"]).unwrap(),
        RecordRequest {
            target: Target::Recorded,
            out: PathBuf::from("v020"),
            same_as: Some(PathBuf::from("earlier")),
        }
    );
    assert_eq!(parse(["record", "092-1440", "--out", "v020"]).unwrap().same_as, None);
}

#[test]
fn a_recording_without_a_folder_to_write_into_is_refused_before_any_detection() {
    assert_eq!(
        parse(["record", "extract"]).unwrap_err().kind(),
        ErrorKind::MissingRequiredArgument
    );
}
