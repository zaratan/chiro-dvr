#[path = "support/agreement.rs"]
mod agreement;

use std::ffi::OsStr;
use std::fs;
use std::path::Path;

use batdetect::options::Options;
use batdetect::process::process;
use batdetect::progress::Silent;
use files::names::Names;
use media::decode::{Decoder, Threads};
use media::frame::{ColorRange, YuvFrame};
use reference::location::{Reference, committed, recorded_video, recording_directories};
use reference::recording::Recording;
use render::config::{RenderConfig, Zoom, ZoomConfig};
use types::language::Language;
use types::rate::FrameRate;
use types::track::Track;

const RECORDED_OUTPUTS: usize = 2;

const FROZEN: [(&str, [usize; 5]); 26] = [
    ("video_089.mp4", [1, 1, 1, 0, 0]),
    ("video_090.mp4", [20, 18, 14, 2, 2]),
    ("video_091.mp4", [21, 21, 21, 0, 0]),
    ("video_092.mp4", [14, 14, 14, 0, 0]),
    ("video_093.mp4", [6, 3, 1, 3, 1]),
    ("video_094.mp4", [14, 13, 10, 1, 0]),
    ("video_095.mp4", [78, 64, 48, 14, 2]),
    ("video_096.mp4", [136, 120, 97, 16, 3]),
    ("video_097.mp4", [149, 131, 111, 18, 4]),
    ("video_098.mp4", [131, 122, 99, 9, 7]),
    ("video_099.mp4", [117, 110, 91, 7, 5]),
    ("video_100.mp4", [98, 93, 80, 5, 4]),
    ("video_101.mp4", [32, 30, 29, 2, 1]),
    ("video_102.mp4", [36, 34, 29, 2, 1]),
    ("video_103.mp4", [10, 10, 9, 0, 0]),
    ("video_120.mp4", [0, 0, 0, 0, 0]),
    ("video_121.mp4", [2, 2, 2, 0, 1]),
    ("video_122.mp4", [0, 0, 0, 0, 0]),
    ("video_123.mp4", [8, 8, 8, 0, 0]),
    ("video_124.mp4", [5, 5, 5, 0, 0]),
    ("video_125.mp4", [47, 43, 36, 4, 2]),
    ("video_126.mp4", [13, 13, 12, 0, 0]),
    ("video_127.mp4", [7, 7, 5, 0, 0]),
    ("video_128.mp4", [5, 5, 4, 0, 0]),
    ("video_129.mp4", [16, 16, 14, 0, 0]),
    ("video_130.mp4", [6, 6, 6, 0, 0]),
];

const UNHELD: [(&str, usize, usize); 5] = [
    ("video_090.mp4", 14, 0),
    ("video_090.mp4", 17, 0),
    ("video_096.mp4", 44, 0),
    ("video_097.mp4", 4, 0),
    ("video_098.mp4", 61, 0),
];

const ZOOM_SWITCHES: [(&str, &[usize], usize, bool); 25] = [
    ("video_090.mp4", &[], 9, true),
    ("video_094.mp4", &[2], 2, true),
    ("video_095.mp4", &[], 25, true),
    ("video_096.mp4", &[], 23, true),
    ("video_096.mp4", &[89], 81, true),
    ("video_097.mp4", &[85], 73, true),
    ("video_097.mp4", &[], 114, true),
    ("video_097.mp4", &[], 117, true),
    ("video_098.mp4", &[], 70, true),
    ("video_098.mp4", &[], 75, true),
    ("video_098.mp4", &[], 89, true),
    ("video_098.mp4", &[], 105, true),
    ("video_098.mp4", &[126], 123, true),
    ("video_098.mp4", &[], 127, true),
    ("video_099.mp4", &[], 28, true),
    ("video_099.mp4", &[], 57, true),
    ("video_099.mp4", &[], 69, true),
    ("video_099.mp4", &[], 87, true),
    ("video_099.mp4", &[90], 88, true),
    ("video_099.mp4", &[], 109, true),
    ("video_100.mp4", &[], 52, true),
    ("video_100.mp4", &[], 87, true),
    ("video_100.mp4", &[], 94, true),
    ("video_121.mp4", &[], 3, true),
    ("video_125.mp4", &[], 35, true),
];

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_by_the_command_keep_their_tracks_and_zooms_frozen_against_the_v020_video_by_video() {
    let directories = recording_directories(Reference::V020).unwrap();
    let out = tempfile::tempdir().unwrap();
    let mut found = Vec::new();
    let mut switches = Vec::new();
    let mut unheld = Vec::new();
    for directory in directories {
        let recording = Recording::read(&directory).unwrap();
        let name = recording.summary.video.name.clone();
        let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
        let job = agreement::job(&recorded_video(&name).unwrap(), &out.path().join(&name));
        let processed = process(
            &job,
            &agreement::DebugLabels,
            &Silent,
            Threads::Automatic,
            agreement::VERSIONS,
            &Options::default(),
        )
        .unwrap();
        let agreement = agreement::agreement(&processed, &recording, rate);
        switches.extend(zoom_switches(&name, &recording, &processed.tracks));
        unheld.extend(not_held_once(&name, &recording, &processed.tracks));
        println!(
            "{name}\treference {}\ttracked {}\tpaired {} (agreeing {})\tlost {}\tadded {}",
            agreement.reference,
            processed.tracks.len(),
            agreement.paired,
            agreement.agreeing,
            agreement.lost,
            agreement.added
        );
        for discrepancy in &agreement.discrepancies {
            println!("{discrepancy:?}");
        }
        found.push((
            name,
            [
                agreement.reference,
                agreement.paired,
                agreement.agreeing,
                agreement.lost,
                agreement.added,
            ],
        ));
    }
    let found: Vec<(&str, [usize; 5])> = found.iter().map(|(name, counts)| (name.as_str(), *counts)).collect();

    let unheld: Vec<(&str, usize, usize)> = unheld
        .iter()
        .map(|(name, id, holders)| (name.as_str(), *id, *holders))
        .collect();
    let switches: Vec<(&str, &[usize], usize, bool)> = switches
        .iter()
        .map(|(name, references, candidate, zoomed)| (name.as_str(), references.as_slice(), *candidate, *zoomed))
        .collect();

    assert_eq!(found, FROZEN);
    assert_eq!(unheld, UNHELD);
    assert_eq!(switches, ZOOM_SWITCHES);
}

#[test]
#[ignore = "local"]
fn the_zooms_recorded_with_the_v011_are_those_its_own_tracks_ask_for() {
    let checked = recording_directories(Reference::V011)
        .unwrap()
        .iter()
        .map(|directory| Recording::read(directory).unwrap())
        .filter(check_recorded_zooms)
        .count();

    assert_eq!(checked, RECORDED_OUTPUTS);
}

type Switch = (String, Vec<usize>, usize, bool);

fn zoom_switches(video: &str, recording: &Recording, tracks: &[Track]) -> Vec<Switch> {
    let auto = ZoomConfig::default();
    tracks
        .iter()
        .filter_map(|candidate| {
            let covering: Vec<&Track> = recording
                .tracks
                .iter()
                .filter(|reference| candidate.points().contains(reference.first()))
                .collect();
            let zoomed = auto.wants(candidate);
            (covering.iter().any(|reference| auto.wants(reference)) != zoomed).then(|| {
                let reference_ids = covering.iter().map(|reference| reference.id()).collect();
                (video.to_owned(), reference_ids, candidate.id(), zoomed)
            })
        })
        .collect()
}

fn not_held_once(video: &str, recording: &Recording, tracks: &[Track]) -> Vec<(String, usize, usize)> {
    recording
        .tracks
        .iter()
        .map(|reference| {
            let holders = tracks
                .iter()
                .filter(|candidate| candidate.points().contains(reference.first()))
                .count();
            (video.to_owned(), reference.id(), holders)
        })
        .filter(|&(_, _, holders)| holders != 1)
        .collect()
}

#[cfg(test)]
fn check_recorded_zooms(recording: &Recording) -> bool {
    let stem = Path::new(&recording.summary.video.name).file_stem().unwrap().to_owned();
    let list = committed(Reference::V011, "zooms").join(stem).with_extension("txt");
    let Ok(names) = fs::read_to_string(&list) else {
        return false;
    };
    let mut recorded: Vec<usize> = names
        .lines()
        .map(|name| name.split('_').nth(1).unwrap().parse().unwrap())
        .collect();
    recorded.sort_unstable();
    let auto = ZoomConfig::default();
    let decided: Vec<usize> = recording
        .tracks
        .iter()
        .filter(|track| auto.wants(track))
        .map(Track::id)
        .collect();
    assert_eq!(recorded, decided, "{}", list.display());
    true
}

#[test]
#[ignore = "local"]
fn the_annotated_103_keeps_the_pts_of_its_decoded_frames_through_its_lost_frames() {
    let directory = recording_directories(Reference::V011)
        .unwrap()
        .into_iter()
        .find(|directory| directory.ends_with("video_103"))
        .unwrap();
    let recording = Recording::read(&directory).unwrap();
    let out = tempfile::tempdir().unwrap();
    let name = recording.summary.video.name.clone();
    let job = agreement::job(&recorded_video(&name).unwrap(), &out.path().join(&name));
    let options = Options {
        render: RenderConfig::default().with_zoom(ZoomConfig::choosing(Zoom::Off)),
        annotated: true,
        ..Options::default()
    };
    process(
        &job,
        &agreement::DebugLabels,
        &Silent,
        Threads::Automatic,
        agreement::VERSIONS,
        &options,
    )
    .unwrap();
    let annotated = Names::new(Language::English).annotated_video(&job.dest, OsStr::new("video_103"));
    let mut decoder = Decoder::open(&annotated, Threads::Automatic).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut pts = Vec::new();
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        pts.push(info.pts());
    }
    let source = recording.frames.pts();
    assert_eq!(source.len(), 8998);
    assert_eq!(pts, source);
}
