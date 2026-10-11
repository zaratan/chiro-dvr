use std::io::{self, Write};
use std::time::Instant;

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
#[ignore = "reference"]
fn the_whole_092_agrees_with_the_reference_on_98_percent_of_its_detections_both_ways() {
    let recording = Recording::read(&committed("video_092_original")).unwrap();
    let path = fixture("video_092_original.mp4").unwrap();

    let start = Instant::now();
    let detected = detect(&path, Threads::Automatic, &DetectConfig::default()).unwrap();
    let seconds = start.elapsed().as_secs_f64();

    let agreement = Agreement::of(&detected, &recording);
    let mut out = io::stdout().lock();
    writeln!(out, "detected in {seconds:.1} s").unwrap();
    agreement.write_summary(&mut out).unwrap();
    agreement.write_groups(&mut out).unwrap();
    assert_eq!(detected.detections().frame_count(), 9011);
    assert!(agreement.reference_rate() >= MIN_RATE && agreement.rust_rate() >= MIN_RATE);
}
