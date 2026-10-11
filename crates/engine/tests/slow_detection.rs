use std::io;

use engine::config::DetectConfig;
use engine::detection::detect;
use media::decode::Threads;
use reference::fixture::fixture;
use reference::location::committed;
use reference::recording::Recording;

#[cfg(test)]
#[path = "support/agreement.rs"]
mod agreement;

use agreement::{Agreement, MIN_RATE};

#[test]
#[ignore = "slow"]
fn the_extract_agrees_with_the_reference_on_98_percent_of_its_detections_both_ways() {
    let recording = Recording::read(&committed("video_092_original_3m24-4m05")).unwrap();
    let path = fixture("video_092_original_3m24-4m05.mp4").unwrap();

    let detected = detect(&path, Threads::Automatic, &DetectConfig::default()).unwrap();

    let agreement = Agreement::of(&detected, &recording);
    let mut out = io::stdout().lock();
    agreement.write_summary(&mut out).unwrap();
    agreement.write_groups(&mut out).unwrap();
    assert_eq!(detected.detections().frame_count(), 1230);
    assert!(agreement.reference_rate() >= MIN_RATE && agreement.rust_rate() >= MIN_RATE);
}
