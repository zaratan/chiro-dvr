use std::path::PathBuf;

use clap::error::ErrorKind;

use super::{Request, command, parse};
use crate::case::Target;

#[test]
fn the_videos_are_named_and_the_frames_folder_is_optional() {
    assert_eq!(
        parse(["compare", "extract"]).unwrap(),
        Request {
            target: Target::Extract,
            frames: None
        }
    );
    assert_eq!(
        parse(["compare", "local", "--frames", "out"]).unwrap(),
        Request {
            target: Target::Recorded,
            frames: Some(PathBuf::from("out"))
        }
    );
    assert_eq!(parse(["compare", "092"]).unwrap().target, Target::Whole);
}

#[test]
fn an_unknown_or_missing_video_set_is_refused() {
    assert_eq!(parse(["compare", "089"]).unwrap_err().kind(), ErrorKind::InvalidValue);
    assert_eq!(
        parse(["compare"]).unwrap_err().kind(),
        ErrorKind::MissingRequiredArgument
    );
}

#[test]
fn the_command_line_is_consistent() {
    command().debug_assert();
}
