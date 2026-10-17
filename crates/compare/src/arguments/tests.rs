use std::path::PathBuf;

use clap::error::ErrorKind;
use reference::location::Reference;

use super::{Request, command, parse};
use crate::case::Target;
use crate::comparison::Folders;

#[test]
fn the_videos_are_named_and_the_frames_folders_are_optional() {
    assert_eq!(
        parse(["compare", "extract"]).unwrap(),
        Request {
            target: Target::Extract,
            reference: Reference::V020,
            folders: Folders::default()
        }
    );
    assert_eq!(
        parse(["compare", "local", "--frames", "out"]).unwrap(),
        Request {
            target: Target::Recorded,
            reference: Reference::V020,
            folders: Folders {
                frames: Some(PathBuf::from("out")),
                same_as: None
            }
        }
    );
    assert_eq!(
        parse(["compare", "local", "--frames", "second", "--same-as", "first"])
            .unwrap()
            .folders,
        Folders {
            frames: Some(PathBuf::from("second")),
            same_as: Some(PathBuf::from("first"))
        }
    );
    assert_eq!(parse(["compare", "092"]).unwrap().target, Target::Whole);
    assert_eq!(parse(["compare", "092-1440"]).unwrap().target, Target::WholeAt1440);
    assert_eq!(parse(["compare", "settings"]).unwrap().target, Target::Settings);
}

#[test]
fn the_v020_is_the_reference_unless_the_v011_is_asked_for_and_no_other_is_known() {
    assert_eq!(
        parse(["compare", "local", "--reference", "v0.1.1"]).unwrap().reference,
        Reference::V011
    );
    assert_eq!(
        parse(["compare", "local", "--reference", "v0.2.0"]).unwrap().reference,
        Reference::V020
    );
    assert_eq!(
        parse(["compare", "local", "--reference", "v0.3.0"]).unwrap_err().kind(),
        ErrorKind::InvalidValue
    );
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
