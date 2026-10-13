use engine::config::DetectConfig;
use render::config::{RenderConfig, Zoom};
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

use super::Options;

#[test]
fn by_default_the_settings_are_those_of_v0_1_1_with_zooms_and_without_annotated_video() {
    let options = Options::default();
    assert_eq!(options.detect, DetectConfig::default());
    assert_eq!(options.track, TrackConfig::default());
    assert_eq!(options.stability, StabilityConfig::default());
    assert_eq!(options.render, RenderConfig::default());
    assert_eq!(options.render.zoom().choice(), Zoom::Auto);
    assert!(!options.annotated);
}
