#[path = "shared/committed.rs"]
mod committed;
#[path = "shared/kept.rs"]
mod kept;

use kept::{Kept, UNCHANGED};
use types::rate::FrameRate;

const FROZEN: [(&str, Kept); 19] = [
    ("video_092_original_3m24-4m05", UNCHANGED),
    ("video_092_original", UNCHANGED),
    ("video_092_original_width_1440", UNCHANGED),
    ("max_area_400", UNCHANGED),
    ("merge_radius_0", UNCHANGED),
    ("min_area_9", UNCHANGED),
    ("noise_factor_0", UNCHANGED),
    ("osd", UNCHANGED),
    ("quick", UNCHANGED),
    ("sigma_0", UNCHANGED),
    ("sigma_3", UNCHANGED),
    ("size_720x540", UNCHANGED),
    ("step_1", UNCHANGED),
    ("threshold_25", UNCHANGED),
    ("width_1440", UNCHANGED),
    ("width_1440_osd", UNCHANGED),
    ("width_480", UNCHANGED),
    ("width_720", UNCHANGED),
    ("window_2s", UNCHANGED),
];

#[test]
fn each_committed_case_keeps_the_detections_and_unstable_spans_of_the_v020_up_to_its_frozen_gaps() {
    let found: Vec<(String, Kept)> = committed::recordings()
        .iter()
        .map(|(name, recording)| {
            let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
            (name.clone(), kept::kept(recording, rate))
        })
        .collect();

    let frozen: Vec<(String, Kept)> = FROZEN.iter().map(|(name, kept)| ((*name).to_owned(), *kept)).collect();
    assert_eq!(found, frozen);
}
