use engine::config::DetectConfig;
use engine::detection::detect;
use engine::input::{Error, Input};
use media::decode::Threads;
use media::frame::ColorRange;
use reference::fixture::fixture;

#[cfg(test)]
fn ranges(name: &str) -> Vec<ColorRange> {
    let input = Input::open(&fixture(name).unwrap(), Threads::Automatic).unwrap();
    let mut ranges = Vec::new();
    input
        .read(|_, frame| {
            ranges.push(frame.range());
            Ok::<(), Error>(())
        })
        .unwrap();
    ranges
}

#[test]
fn a_yuvj420p_video_is_read_to_its_end_since_the_detection_reads_raw_luma() {
    assert_eq!(ranges("full_range.mp4"), [ColorRange::Full; 2]);
}

#[test]
fn a_video_that_turns_to_full_range_midway_is_read_to_its_end() {
    let found = ranges("full_range_midway.mp4");

    assert_eq!(found.len(), 4);
    assert_eq!(found[2..], [ColorRange::Full; 2]);
    assert!(found[..2].iter().all(|&range| range != ColorRange::Full), "{found:?}");
}

#[test]
fn the_same_stream_marked_full_range_gives_the_same_detections_bit_for_bit() {
    let limited = detect(
        &fixture("spot.mp4").unwrap(),
        Threads::Automatic,
        &DetectConfig::default(),
    )
    .unwrap();
    let full = detect(
        &fixture("spot_full_range.mp4").unwrap(),
        Threads::Automatic,
        &DetectConfig::default(),
    )
    .unwrap();

    assert!(ranges("spot.mp4").iter().all(|&range| range != ColorRange::Full));
    assert_eq!(ranges("spot_full_range.mp4"), [ColorRange::Full; 40]);
    assert!(limited.detections().detection_count() > 0);
    assert_eq!(limited.detections(), full.detections());
}
