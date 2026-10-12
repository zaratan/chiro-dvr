use std::path::PathBuf;

use clap::error::ErrorKind;

use super::{Request, parse};

#[test]
fn several_videos_and_the_long_output_option_are_read() {
    let request = parse(["batdetect-rs", "a.mp4", "nuit", "--output", "out"]).unwrap();
    assert_eq!(
        request,
        Request {
            inputs: vec![PathBuf::from("a.mp4"), PathBuf::from("nuit")],
            output: PathBuf::from("out"),
        }
    );
}

#[test]
fn the_output_option_may_come_first() {
    let request = parse(["batdetect-rs", "-o", "out", "a.mp4"]).unwrap();
    assert_eq!(request.output, PathBuf::from("out"));
}

#[test]
fn a_missing_output_folder_is_a_usage_error() {
    let error = parse(["batdetect-rs", "a.mp4"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
    assert_eq!(error.exit_code(), 2);
}

#[test]
fn an_output_folder_without_video_is_a_usage_error() {
    let error = parse(["batdetect-rs", "-o", "out"]).unwrap_err();
    assert_eq!(error.kind(), ErrorKind::MissingRequiredArgument);
}
