#[path = "shared/committed.rs"]
mod committed;
#[path = "shared/comparison.rs"]
mod comparison;

use comparison::Tracks;
use types::rate::FrameRate;
use types::track::Track;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Identical(usize);

const FROZEN: [(&str, Tracks, Identical); 19] = [
    ("video_092_original_3m24-4m05", Tracks::same(4), Identical(4)),
    ("video_092_original", Tracks::same(14), Identical(14)),
    ("video_092_original_width_1440", Tracks::same(14), Identical(14)),
    ("max_area_400", Tracks::same(4), Identical(4)),
    ("merge_radius_0", Tracks::same(4), Identical(4)),
    ("min_area_9", Tracks::same(5), Identical(5)),
    ("noise_factor_0", Tracks::same(4), Identical(4)),
    ("osd", Tracks::same(4), Identical(4)),
    ("quick", Tracks::same(4), Identical(4)),
    ("sigma_0", Tracks::same(30), Identical(30)),
    ("sigma_3", Tracks::same(4), Identical(4)),
    ("size_720x540", Tracks::same(4), Identical(4)),
    ("step_1", Tracks::same(5), Identical(5)),
    ("threshold_25", Tracks::same(4), Identical(4)),
    ("width_1440", Tracks::same(4), Identical(4)),
    ("width_1440_osd", Tracks::same(4), Identical(4)),
    ("width_480", Tracks::same(4), Identical(4)),
    ("width_720", Tracks::same(5), Identical(5)),
    ("window_2s", Tracks::same(4), Identical(4)),
];

#[test]
fn each_committed_case_tracks_its_v020_detections_into_its_frozen_tracks_numbered_from_one() {
    let found: Vec<(String, Tracks, Identical)> = committed::recordings()
        .iter()
        .map(|(name, recording)| {
            let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
            let found = comparison::compare(recording, rate).unwrap();
            println!(
                "{name} ({}): {:?}, {} identical, {} with other frames, largest gap {:e}",
                found.video,
                found.tracks(),
                found.identical,
                found.different_frames,
                found.largest_gap
            );
            for discrepancy in &found.discrepancies {
                println!("  {discrepancy:?}");
            }
            let numbers: Vec<usize> = found.tracked.iter().map(Track::id).collect();
            assert_eq!(numbers, (1..=found.tracked.len()).collect::<Vec<_>>(), "{name}");
            (name.clone(), found.tracks(), Identical(found.identical))
        })
        .collect();

    let frozen: Vec<(String, Tracks, Identical)> = FROZEN
        .iter()
        .map(|(name, tracks, identical)| ((*name).to_owned(), *tracks, *identical))
        .collect();
    assert_eq!(found, frozen);
}
