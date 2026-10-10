use std::sync::Once;

use ffmpeg_next as ffmpeg;

static INIT: Once = Once::new();

pub(crate) fn ffmpeg() {
    INIT.call_once(|| {
        ffmpeg::util::error::register_all();
        ffmpeg::util::log::set_level(ffmpeg::util::log::Level::Quiet);
    });
}
