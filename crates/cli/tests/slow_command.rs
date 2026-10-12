#[path = "support/agreement.rs"]
mod agreement;

use std::fs;
use std::sync::OnceLock;

use batdetect::process::{Processed, process};
use media::decode::Threads;
use reference::fixture::fixture;
use reference::location::committed;
use reference::recording::Recording;
use types::frame::FrameIndex;
use types::language::Language;
use types::rate::FrameRate;

const EXTRACT: &str = "video_092_original_3m24-4m05";
const EXTRACT_FIRST_SOURCE_FRAME: usize = 6150;
const BAT_CONFIRMED_AT_S: f64 = 3.0 * 60.0 + 58.0;

#[cfg(test)]
fn extract() -> &'static (Recording, FrameRate, Processed, tempfile::TempDir) {
    static EXTRACT_RUN: OnceLock<(Recording, FrameRate, Processed, tempfile::TempDir)> = OnceLock::new();
    EXTRACT_RUN.get_or_init(|| {
        let recording = Recording::read(&committed(EXTRACT)).unwrap();
        let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
        let out = tempfile::tempdir().unwrap();
        let video = fixture(&format!("{EXTRACT}.mp4")).unwrap();
        let job = agreement::job(&video, &out.path().join(EXTRACT));
        let processed = process(&job, Language::English, Threads::Automatic, agreement::VERSIONS).unwrap();
        (recording, rate, processed, out)
    })
}

#[test]
#[ignore = "slow"]
fn the_extract_gives_the_four_tracks_of_the_reference_in_its_table() {
    let (recording, rate, processed, _out) = extract();
    let found = agreement::agreement(processed.clone(), recording, *rate);
    assert_eq!((found.reference, found.paired, found.agreeing), (4, 4, 4));
    assert_eq!((found.lost, found.added), (0, 0));
    assert_eq!(found.discrepancies, []);
    let table = fs::read_to_string(&found.processed.csv).unwrap();
    let hits: Vec<usize> = table
        .lines()
        .skip(1)
        .map(|line| line.split(',').nth(5).unwrap().parse().unwrap())
        .collect();
    let expected: Vec<usize> = recording.rows.iter().map(|row| row.hits).collect();
    assert_eq!(hits, expected);
}

#[test]
#[ignore = "slow"]
fn the_bat_the_naturalist_confirmed_at_3_58_is_tracked() {
    let (_recording, rate, processed, _out) = extract();
    let extract_start = rate.seconds(FrameIndex::new(EXTRACT_FIRST_SOURCE_FRAME));
    let confirmed = BAT_CONFIRMED_AT_S - extract_start;
    let tracked = processed.tracks.iter().any(|track| {
        let (start, end) = (rate.seconds(track.first().frame), rate.seconds(track.last().frame));
        start - 1.0 <= confirmed && confirmed <= end + 1.0
    });
    assert!(tracked);
}
