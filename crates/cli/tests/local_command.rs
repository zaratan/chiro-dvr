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

const fn agreeing(tracks: usize) -> [usize; 5] {
    [tracks, tracks, tracks, 0, 0]
}

const fn rejoined(reference: usize, joins: usize, tracks_joined: usize) -> [usize; 5] {
    [
        reference,
        reference - joins,
        reference - joins - tracks_joined,
        joins,
        0,
    ]
}

const FROZEN: [(&str, [usize; 5]); 26] = [
    ("video_089.mp4", agreeing(1)),
    ("video_090.mp4", agreeing(20)),
    ("video_091.mp4", agreeing(21)),
    ("video_092.mp4", agreeing(14)),
    ("video_093.mp4", rejoined(6, 2, 2)),
    ("video_094.mp4", rejoined(14, 1, 1)),
    ("video_095.mp4", rejoined(78, 11, 11)),
    ("video_096.mp4", rejoined(136, 13, 12)),
    ("video_097.mp4", rejoined(149, 15, 11)),
    ("video_098.mp4", rejoined(131, 8, 7)),
    ("video_099.mp4", rejoined(117, 7, 7)),
    ("video_100.mp4", rejoined(98, 5, 5)),
    ("video_101.mp4", rejoined(32, 1, 1)),
    ("video_102.mp4", rejoined(36, 1, 1)),
    ("video_103.mp4", agreeing(10)),
    ("video_120.mp4", agreeing(0)),
    ("video_121.mp4", agreeing(2)),
    ("video_122.mp4", agreeing(0)),
    ("video_123.mp4", agreeing(8)),
    ("video_124.mp4", agreeing(5)),
    ("video_125.mp4", rejoined(47, 3, 3)),
    ("video_126.mp4", agreeing(13)),
    ("video_127.mp4", agreeing(7)),
    ("video_128.mp4", agreeing(5)),
    ("video_129.mp4", agreeing(16)),
    ("video_130.mp4", agreeing(6)),
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

    assert_eq!(found, FROZEN);
    assert!(unheld.is_empty(), "{unheld:?}");
    assert!(switches.is_empty(), "{switches:?}");
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
