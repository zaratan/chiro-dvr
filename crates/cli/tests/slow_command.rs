#[path = "support/agreement.rs"]
mod agreement;
#[path = "support/clips.rs"]
mod clips;

use std::fs;
use std::sync::OnceLock;

use batdetect::options::Options;
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
        let processed = process(
            &job,
            Language::English,
            &agreement::DebugLabels,
            Threads::Automatic,
            agreement::VERSIONS,
            Options::default(),
        )
        .unwrap();
        (recording, rate, processed, out)
    })
}

#[test]
#[ignore = "slow"]
fn the_extract_gives_the_four_tracks_of_the_reference_in_its_table() {
    let (recording, rate, processed, _out) = extract();
    let found = agreement::agreement(processed, recording, *rate);
    assert_eq!((found.reference, found.paired, found.agreeing), (4, 4, 4));
    assert_eq!((found.lost, found.added), (0, 0));
    assert_eq!(found.discrepancies, []);
    let table = fs::read_to_string(&processed.csv).unwrap();
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

#[test]
#[ignore = "slow"]
fn the_extract_gives_the_four_clips_of_the_reference_with_their_names_and_lengths() {
    let (_recording, _rate, _processed, out) = extract();
    let lengths = clips::lengths(&out.path().join(EXTRACT).join("clips"));
    let expected = [
        ("track_01_0m04s.mp4", 380),
        ("track_02_0m23s.mp4", 108),
        ("track_03_0m26s.mp4", 115),
        ("track_04_0m32s.mp4", 117),
    ]
    .map(|(name, frames)| (name.to_owned(), frames));
    assert_eq!(lengths, expected);
}
