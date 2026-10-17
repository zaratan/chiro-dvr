#[path = "support/frozen.rs"]
mod frozen;

use std::sync::OnceLock;

use reference::fixture::fixture;

use crate::frozen::{Drawing, Frozen, clip, tracked};

const EXTRACT: &str = "video_092_original_3m24-4m05.mp4";
const SLOW_TRACK_OF_3_29_S: f64 = 4.80;
const FAST_TRACK_OF_3_48_S: f64 = 23.48;
const SLOW_TRACK_CLIP: Frozen = Frozen {
    frames: 380,
    fingerprint: 18_137_511_074_968_367_599,
};
const FAST_TRACK_CLIP: Frozen = Frozen {
    frames: 108,
    fingerprint: 3_750_025_716_088_547_530,
};

#[cfg(test)]
fn extract() -> &'static Drawing {
    static DRAWING: OnceLock<Drawing> = OnceLock::new();
    DRAWING.get_or_init(|| tracked(&fixture(EXTRACT).unwrap()))
}

#[test]
#[ignore = "slow"]
fn the_clip_of_the_slow_track_of_3_29_judged_as_good_as_v0_1_1_draws_the_same_frames() {
    assert_eq!(extract().frozen(SLOW_TRACK_OF_3_29_S, clip), SLOW_TRACK_CLIP);
}

#[test]
#[ignore = "slow"]
fn the_clip_of_the_fast_pass_of_3_48_judged_as_good_as_v0_1_1_draws_the_same_frames() {
    assert_eq!(extract().frozen(FAST_TRACK_OF_3_48_S, clip), FAST_TRACK_CLIP);
}
