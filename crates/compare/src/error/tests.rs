use engine::settings::{self, Bound, Setting};

use super::Error;

#[test]
fn a_recorded_setting_the_engine_refuses_names_its_video_then_the_bound_it_breaks() {
    let error = Error::Settings {
        video: "video.mp4".to_owned(),
        source: settings::Error::Setting {
            setting: Setting::Threshold,
            value: -1.0,
            bound: Bound::Positive,
        },
    };

    assert_eq!(
        error.to_string(),
        "video.mp4: the recorded detection settings cannot configure the engine"
    );
    assert_eq!(
        std::error::Error::source(&error).map(ToString::to_string).as_deref(),
        Some("threshold must be a finite number > 0, not -1")
    );
}
