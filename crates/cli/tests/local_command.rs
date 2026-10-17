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
use reference::track_matching::match_tracks;
use render::config::{RenderConfig, Zoom, ZoomConfig};
use types::language::Language;
use types::rate::FrameRate;
use types::track::Track;

const RECORDED_OUTPUTS: usize = 2;

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_by_the_command_keep_their_tracks_and_zooms_frozen_against_the_v020_video_by_video() {
    let directories = recording_directories(Reference::V020).unwrap();
    let out = tempfile::tempdir().unwrap();
    let mut found = Vec::new();
    let mut switches = Vec::new();
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
    for switch in &switches {
        println!(
            "zoom switch {} reference #{} (area {:.2}, amplitude {:.2}) -> #{} (area {:.2}, amplitude {:.2}) zoomed {}",
            switch.video,
            switch.reference_id,
            switch.reference.0,
            switch.reference.1,
            switch.candidate_id,
            switch.candidate.0,
            switch.candidate.1,
            switch.zoomed
        );
    }
    let found: Vec<(&str, [usize; 5])> = found.iter().map(|(name, counts)| (name.as_str(), *counts)).collect();
    let agreeing = |tracks: usize| [tracks, tracks, tracks, 0, 0];

    assert_eq!(
        found,
        [
            ("video_089.mp4", agreeing(1)),
            ("video_090.mp4", [20, 18, 14, 2, 2]),
            ("video_091.mp4", agreeing(21)),
            ("video_092.mp4", agreeing(14)),
            ("video_093.mp4", [6, 6, 6, 0, 1]),
            ("video_094.mp4", [14, 14, 12, 0, 0]),
            ("video_095.mp4", [78, 78, 70, 0, 2]),
            ("video_096.mp4", [136, 135, 122, 1, 4]),
            ("video_097.mp4", [149, 148, 137, 1, 5]),
            ("video_098.mp4", [131, 130, 114, 1, 8]),
            ("video_099.mp4", [117, 117, 105, 0, 5]),
            ("video_100.mp4", [98, 98, 90, 0, 4]),
            ("video_101.mp4", [32, 31, 31, 1, 1]),
            ("video_102.mp4", [36, 36, 32, 0, 2]),
            ("video_103.mp4", [10, 10, 9, 0, 0]),
            ("video_120.mp4", agreeing(0)),
            ("video_121.mp4", [2, 2, 2, 0, 1]),
            ("video_122.mp4", agreeing(0)),
            ("video_123.mp4", agreeing(8)),
            ("video_124.mp4", agreeing(5)),
            ("video_125.mp4", [47, 46, 42, 1, 2]),
            ("video_126.mp4", [13, 13, 12, 0, 0]),
            ("video_127.mp4", [7, 7, 5, 0, 0]),
            ("video_128.mp4", [5, 5, 4, 0, 0]),
            ("video_129.mp4", [16, 16, 14, 0, 0]),
            ("video_130.mp4", agreeing(6)),
        ]
    );
    assert_eq!(
        identified(&switches),
        [
            ("video_094.mp4", 2, 2, true),
            ("video_097.mp4", 85, 84, true),
            ("video_099.mp4", 90, 94, true),
        ]
    );
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

#[derive(Debug)]
struct Switch {
    video: String,
    reference_id: usize,
    candidate_id: usize,
    zoomed: bool,
    reference: (f64, f64),
    candidate: (f64, f64),
}

fn identified(switches: &[Switch]) -> Vec<(&str, usize, usize, bool)> {
    switches
        .iter()
        .map(|switch| {
            (
                switch.video.as_str(),
                switch.reference_id,
                switch.candidate_id,
                switch.zoomed,
            )
        })
        .collect()
}

#[cfg(test)]
fn zoom_switches(video: &str, recording: &Recording, tracks: &[Track]) -> Vec<Switch> {
    let auto = ZoomConfig::default();
    let measures = |track: &Track| (track.max_area(), track.peak_amplitude());
    match_tracks(&recording.tracks, tracks)
        .pairs
        .into_iter()
        .filter(|(reference, candidate)| auto.wants(reference) != auto.wants(candidate))
        .map(|(reference, candidate)| Switch {
            video: video.to_owned(),
            reference_id: reference.id(),
            candidate_id: candidate.id(),
            zoomed: auto.wants(candidate),
            reference: measures(reference),
            candidate: measures(candidate),
        })
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
