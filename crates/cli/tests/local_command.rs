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
use reference::track_matching::{DiscrepancyKind, match_tracks};
use render::config::{RenderConfig, Zoom, ZoomConfig};
use types::language::Language;
use types::rate::FrameRate;
use types::track::Track;

const RECORDED_OUTPUTS: usize = 2;

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_give_the_tracks_of_the_reference_but_for_the_known_divergences() {
    let directories = recording_directories(Reference::V011).unwrap();
    assert_eq!(directories.len(), 26);
    let out = tempfile::tempdir().unwrap();
    let (mut reference, mut paired, mut agreeing, mut lost, mut added) = (0, 0, 0, 0, 0);
    let mut discrepancies = Vec::new();
    let mut switches = Vec::new();
    let mut checked = 0;
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
        let found = agreement::agreement(&processed, &recording, rate);
        switches.extend(zoom_switches(&name, &recording, &processed.tracks));
        checked += usize::from(check_recorded_zooms(&recording));
        println!(
            "{name}\treference {}\ttracked {}\tpaired {} (agreeing {})\tlost {}\tadded {}",
            found.reference,
            processed.tracks.len(),
            found.paired,
            found.agreeing,
            found.lost,
            found.added
        );
        reference += found.reference;
        paired += found.paired;
        agreeing += found.agreeing;
        lost += found.lost;
        added += found.added;
        discrepancies.extend(found.discrepancies);
    }
    println!("total\treference {reference}\tpaired {paired} (agreeing {agreeing})\tlost {lost}\tadded {added}");
    for discrepancy in &discrepancies {
        println!("{discrepancy:?}");
    }
    assert_eq!(reference, 966);
    assert_eq!((paired, agreeing, lost, added), (963, 958, 3, 9));
    let found: Vec<(&str, usize, DiscrepancyKind)> = discrepancies
        .iter()
        .map(|discrepancy| (discrepancy.video.as_str(), discrepancy.frame.get(), discrepancy.kind))
        .collect();
    assert_eq!(found, known_divergences());
    assert_eq!(checked, RECORDED_OUTPUTS);
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
    let zoom_flips: Vec<(&str, usize, bool)> = switches
        .iter()
        .map(|switch| (switch.video.as_str(), switch.reference_id, switch.zoomed))
        .collect();
    assert_eq!(zoom_flips, known_zoom_switches());
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

#[cfg(test)]
fn known_zoom_switches() -> Vec<(&'static str, usize, bool)> {
    vec![("video_097.mp4", 88, true)]
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

#[cfg(test)]
fn known_divergences() -> Vec<(&'static str, usize, DiscrepancyKind)> {
    let points = |id, reference_points, candidate_points| DiscrepancyKind::Points {
        reference_id: id,
        candidate_id: id,
        reference_points,
        candidate_points,
    };
    let missed = |reference_id| DiscrepancyKind::Missed { reference_id };
    let extra = |candidate_id| DiscrepancyKind::Extra { candidate_id };
    vec![
        ("video_094.mp4", 3069, missed(3)),
        ("video_094.mp4", 3095, extra(3)),
        ("video_097.mp4", 612, missed(8)),
        ("video_097.mp4", 619, extra(8)),
        ("video_097.mp4", 4872, points(88, 86, 24)),
        ("video_097.mp4", 4906, extra(90)),
        ("video_098.mp4", 176, extra(4)),
        ("video_098.mp4", 1245, missed(20)),
        ("video_099.mp4", 429, points(14, 75, 119)),
        ("video_099.mp4", 509, points(16, 60, 16)),
        ("video_099.mp4", 538, points(17, 127, 50)),
        ("video_099.mp4", 583, extra(19)),
        ("video_099.mp4", 838, extra(30)),
        ("video_100.mp4", 2890, extra(34)),
        ("video_100.mp4", 2894, extra(36)),
        ("video_126.mp4", 2754, points(8, 32, 27)),
        ("video_126.mp4", 2776, extra(9)),
    ]
}
