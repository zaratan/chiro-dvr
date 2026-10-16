use std::path::Path;

use super::{location, partial};

#[test]
fn a_location_holds_the_bytes_of_the_path() {
    assert_eq!(location(Path::new("out/clip.mp4")).unwrap().as_bytes(), b"out/clip.mp4");
    assert_eq!(location(Path::new("out/cl\0ip.mp4")), None);
}

#[cfg(windows)]
#[test]
fn a_path_that_is_not_unicode_has_no_location_under_windows() {
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    let lone_surrogate = OsString::from_wide(&[0xD800, u16::from(b'v')]);
    assert_eq!(location(Path::new(&lone_surrogate)), None);
}

#[cfg(unix)]
#[test]
fn a_location_keeps_the_exact_bytes_of_a_path_that_is_not_utf8() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let path = Path::new(OsStr::from_bytes(b"sortie_\xe9/clip.mp4"));
    assert_eq!(location(path).unwrap().as_bytes(), b"sortie_\xe9/clip.mp4");
}

#[test]
fn the_partial_file_is_an_empty_dot_file_beside_its_destination_with_a_unique_part() {
    let folder = tempfile::tempdir().unwrap();
    let partial = partial(&folder.path().join("clip.mp4")).unwrap().unwrap();
    assert_eq!(partial.parent(), Some(folder.path()));
    assert_eq!(std::fs::read(&partial).unwrap(), b"");
    let name = partial.file_name().unwrap().to_str().unwrap();
    let unique = name
        .strip_prefix(".clip.mp4.")
        .and_then(|rest| rest.strip_suffix(".partial"))
        .unwrap();
    assert_eq!(unique.len(), 6, "{name}");
}

#[test]
fn two_partial_files_of_the_same_clip_are_two_files() {
    let folder = tempfile::tempdir().unwrap();
    let clip = folder.path().join("clip.mp4");
    let (first, second) = (partial(&clip).unwrap().unwrap(), partial(&clip).unwrap().unwrap());
    assert_ne!(first.to_path_buf(), second.to_path_buf());
}

#[test]
fn a_path_without_a_file_name_has_no_partial_file() {
    assert!(partial(Path::new("out/..")).is_none());
}
