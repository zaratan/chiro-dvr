use std::path::Path;

use super::for_ffmpeg;

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
