use render::config::{RenderConfig, Zoom};

use super::Options;

#[test]
fn by_default_tracks_are_zoomed_as_v0_1_1_did_and_no_annotated_video_is_written() {
    let options = Options::default();
    assert_eq!(options.render, RenderConfig::default());
    assert_eq!(options.render.zoom().choice(), Zoom::Auto);
    assert!(!options.annotated);
}
