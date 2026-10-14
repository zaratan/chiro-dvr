use std::io;

use ffmpeg_next as ffmpeg;
use ffmpeg_next::util::error::{EACCES, EAGAIN, ENOENT, ENOSPC, EPERM, EROFS};

use super::LibraryError;
use crate::init;

#[test]
fn errors_carry_the_ffmpeg_message_once_initialised() {
    init::ffmpeg();
    assert_eq!(
        LibraryError::new(ffmpeg::Error::InvalidData).to_string(),
        "Invalid data found when processing input"
    );
}

#[test]
fn the_errno_of_a_system_error_gives_its_kind() {
    let kind = |errno| LibraryError::new(ffmpeg::Error::Other { errno }).io_kind();
    assert_eq!(kind(ENOENT), Some(io::ErrorKind::NotFound));
    assert_eq!(kind(EACCES), Some(io::ErrorKind::PermissionDenied));
    assert_eq!(kind(EPERM), Some(io::ErrorKind::PermissionDenied));
    assert_eq!(kind(ENOSPC), Some(io::ErrorKind::StorageFull));
    assert_eq!(kind(EROFS), Some(io::ErrorKind::ReadOnlyFilesystem));
    assert_eq!(kind(EAGAIN), None);
}

#[test]
fn an_error_of_ffmpeg_itself_has_no_system_kind() {
    assert_eq!(LibraryError::new(ffmpeg::Error::InvalidData).io_kind(), None);
}
