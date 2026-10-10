use ffmpeg_next as ffmpeg;

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
