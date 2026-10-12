use std::path::Path;

use super::{for_ffmpeg, partial};

#[test]
fn a_plain_path_goes_through() {
    assert_eq!(for_ffmpeg(Path::new("out/clip.mp4")), Some("out/clip.mp4"));
}

#[test]
fn a_nul_byte_is_refused() {
    assert_eq!(for_ffmpeg(Path::new("out/cl\0ip.mp4")), None);
}

#[cfg(unix)]
#[test]
fn a_path_that_is_not_utf8_is_refused() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    assert_eq!(for_ffmpeg(Path::new(OsStr::from_bytes(b"\xff.mp4"))), None);
}

#[test]
fn the_partial_file_is_hidden_beside_its_destination() {
    assert_eq!(
        partial(Path::new("out/clip.mp4")),
        Some(Path::new("out/.clip.mp4.partial").to_path_buf())
    );
    assert_eq!(
        partial(Path::new("clip.mp4")),
        Some(Path::new(".clip.mp4.partial").to_path_buf())
    );
}

#[test]
fn a_path_without_a_file_name_has_no_partial_file() {
    assert_eq!(partial(Path::new("out/..")), None);
}
