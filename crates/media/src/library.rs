use std::borrow::Cow;
use std::ffi::CStr;

use ffmpeg_next as ffmpeg;

#[cfg(test)]
mod tests;

pub fn version() -> Cow<'static, str> {
    // SAFETY: av_version_info takes no argument and returns a pointer to a static string.
    let pointer = unsafe { ffmpeg::ffi::av_version_info() };
    // SAFETY: that string is NUL-terminated and lives as long as the program, as FFmpeg documents.
    let text: &'static CStr = unsafe { CStr::from_ptr(pointer) };
    text.to_string_lossy()
}

pub fn license() -> &'static str {
    ffmpeg::codec::license()
}
