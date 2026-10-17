#[path = "support/agreement.rs"]
mod agreement;
#[path = "support/clips.rs"]
mod clips;
#[path = "support/panics.rs"]
mod panics;

use std::fs;
use std::panic;
use std::sync::OnceLock;

use batdetect::options::Options;
use batdetect::process::{Processed, process};
use batdetect::progress::Silent;
use files::table::{COLUMNS, Column};
use media::decode::Threads;
use reference::fixture::fixture;
use reference::location::{Reference, committed};
use reference::recording::Recording;
use types::frame::FrameIndex;
use types::rate::FrameRate;

use crate::panics::panicked;

const EXTRACT: &str = "video_092_original_3m24-4m05";
const EXTRACT_FIRST_SOURCE_FRAME: usize = 6150;
const BAT_CONFIRMED_AT_S: f64 = 3.0 * 60.0 + 58.0;

struct ExtractRun {
    recording: Recording,
    rate: FrameRate,
    processed_in_removed_folder: Processed,
    table: Result<String, String>,
    clips: Result<Vec<(String, usize)>, String>,
}

#[cfg(test)]
fn extract() -> &'static ExtractRun {
    static EXTRACT_RUN: OnceLock<ExtractRun> = OnceLock::new();
    EXTRACT_RUN.get_or_init(|| {
        let recording = Recording::read(&committed(Reference::V011, EXTRACT)).unwrap();
        let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
        let out = tempfile::tempdir().unwrap();
        let video = fixture(&format!("{EXTRACT}.mp4")).unwrap();
        let job = agreement::job(&video, &out.path().join(EXTRACT));
        let processed = process(
            &job,
            &agreement::DebugLabels,
            &Silent,
            Threads::Automatic,
            agreement::VERSIONS,
            &Options::default(),
        )
        .unwrap();
        let table = processed
            .csv
            .as_ref()
            .ok_or_else(|| "no tracks table".to_owned())
            .and_then(|csv| fs::read_to_string(csv).map_err(|error| error.to_string()));
        let folder = out.path().join(EXTRACT).join("clips");
        let clips = panic::catch_unwind(|| clips::lengths(&folder)).map_err(|payload| panicked(&*payload));
        ExtractRun {
            recording,
            rate,
            processed_in_removed_folder: processed,
            table,
            clips,
        }
    })
}

#[test]
#[ignore = "slow"]
fn the_extract_gives_the_four_tracks_of_the_reference_in_its_table() {
    let run = extract();
    let found = agreement::agreement(&run.processed_in_removed_folder, &run.recording, run.rate);
    assert_eq!((found.reference, found.paired, found.agreeing), (4, 4, 4));
    assert_eq!((found.lost, found.added), (0, 0));
    assert_eq!(found.discrepancies, []);
    let hits = COLUMNS.iter().position(|&column| column == Column::Hits).unwrap();
    let hits: Vec<usize> = run
        .table
        .as_ref()
        .unwrap()
        .lines()
        .skip(1)
        .map(|line| line.split(',').nth(hits).unwrap().parse().unwrap())
        .collect();
    let expected: Vec<usize> = run
        .recording
        .rows
        .as_deref()
        .unwrap()
        .iter()
        .map(|row| row.hits)
        .collect();
    assert_eq!(hits, expected);
}

#[test]
#[ignore = "slow"]
fn the_bat_the_naturalist_confirmed_at_3_58_is_tracked() {
    let run = extract();
    let extract_start = run.rate.seconds(FrameIndex::new(EXTRACT_FIRST_SOURCE_FRAME));
    let confirmed = BAT_CONFIRMED_AT_S - extract_start;
    let tracked = run.processed_in_removed_folder.tracks.iter().any(|track| {
        let (start, end) = (
            run.rate.seconds(track.first().frame),
            run.rate.seconds(track.last().frame),
        );
        start - 1.0 <= confirmed && confirmed <= end + 1.0
    });
    assert!(tracked);
}

#[test]
#[ignore = "slow"]
fn the_extract_gives_the_four_clips_of_the_reference_with_their_names_and_lengths() {
    let expected = [
        ("track_01_0m04s.mp4", 380),
        ("track_02_0m23s.mp4", 108),
        ("track_03_0m26s.mp4", 115),
        ("track_04_0m32s.mp4", 117),
    ]
    .map(|(name, frames)| (name.to_owned(), frames));
    assert_eq!(extract().clips.as_ref().unwrap(), &expected);
}
