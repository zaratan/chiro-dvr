use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use batdetect::options::Options;
use batdetect::process::{Tracked, tracked};
use engine::detection::detect;
use media::decode::{Decoder, Threads};
use media::fingerprint;
use media::frame::{ColorRange, YuvFrame};
use reference::location::{Reference, recorded_video, recording_directories};
use reference::recording::Recording;
use render::config::RenderConfig;
use render::source::{Error, Origin, Source};
use render::windows::{Group, Window, clip_windows, groups};
use types::span::Span;
use types::timeline::Timeline;
use types::track::Track;

const RECORDED_VIDEOS: usize = 26;
const DAMAGED: [(&str, usize); 3] = [("video_103", 0), ("video_123", 0), ("video_124", 0)];
type Frames = (usize, usize);
type Diverging = (String, usize, usize, usize, usize);

#[derive(Debug)]
struct Engine {
    video: PathBuf,
    timeline: Timeline,
    tracks: Vec<Track>,
    damaged: Vec<Span>,
    source: Source,
}

#[cfg(test)]
fn through_the_engine(name: &str) -> Engine {
    let video = recorded_video(&format!("{name}.mp4")).unwrap();
    let options = Options::default();
    let detected = detect(&video, Threads::Automatic, &options.detect).unwrap();
    let Tracked {
        timeline,
        exclusion,
        tracks,
    } = tracked(&detected, &options).unwrap();
    let errors = detected.damage().error_frames();
    let source = Source::new(&video, &timeline, &detected.reading().key_frames(), errors).unwrap();
    Engine {
        video,
        timeline,
        tracks,
        damaged: exclusion.damaged,
        source,
    }
}

#[cfg(test)]
fn fingerprints(path: &Path) -> Vec<u64> {
    let mut decoder = Decoder::open(path, Threads::Exactly(NonZeroUsize::MIN)).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut fingerprints = Vec::new();
    while decoder.next(&mut frame).unwrap().is_some() {
        fingerprints.push(fingerprint::of(&frame));
    }
    fingerprints
}

#[cfg(test)]
fn windows_of(tracks: &[Track], timeline: &Timeline) -> Vec<Window> {
    clip_windows(tracks, timeline.frame_count(), RenderConfig::default(), timeline.rate())
}

#[test]
#[ignore = "local"]
fn groups_over_damaged_spans_serve_the_pixels_of_a_single_threaded_reading_from_the_key_frames_of_the_engine() {
    for (name, expected_restarts) in DAMAGED {
        let engine = through_the_engine(name);
        let sequential = fingerprints(&engine.video);
        assert_eq!(sequential.len(), engine.timeline.frame_count(), "{name}");
        let windows = windows_of(&engine.tracks, &engine.timeline);
        let touched: Vec<Span> = groups(&windows)
            .iter()
            .map(Group::span)
            .filter(|span| {
                engine
                    .damaged
                    .iter()
                    .any(|damaged| damaged.intersection(*span).is_some())
            })
            .collect();
        let mut restarts = 0;
        for span in &touched {
            let mut served = Vec::new();
            let origin = engine
                .source
                .serve(*span, |frame, pixels| {
                    served.push(fingerprint::of(pixels) == sequential[frame.get()]);
                    Ok::<(), Error>(())
                })
                .unwrap();
            assert_eq!(served.len(), span.frame_count(), "{name} {span:?}");
            assert!(served.iter().all(|same| *same), "{name} {span:?}");
            restarts += usize::from(matches!(origin, Origin::Start(_)));
            println!("{name} {span:?}: {origin:?}");
        }
        println!(
            "{name}: {} tracks, {} groups over damaged spans, {restarts} read from the start",
            engine.tracks.len(),
            touched.len()
        );
        assert!(!touched.is_empty(), "{name}");
        assert_eq!(restarts, expected_restarts, "{name}");
    }
}

#[test]
#[ignore = "local"]
fn the_windows_of_the_twenty_six_videos_keep_their_gaps_to_those_of_the_v020_tracks_frozen_where_flights_are_rejoined()
{
    let mut videos = Vec::new();
    let mut diverging: Vec<Diverging> = Vec::new();
    for directory in recording_directories(Reference::V020).unwrap() {
        let recording = Recording::read(&directory).unwrap();
        let name = Path::new(&recording.summary.video.name)
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let engine = through_the_engine(&name);
        let frames = |windows: Vec<Window>| -> Vec<Frames> {
            windows
                .iter()
                .map(|window| (window.span.first().get(), window.span.last().get()))
                .collect()
        };
        let ours = frames(windows_of(&engine.tracks, &engine.timeline));
        let theirs = frames(windows_of(&recording.tracks, &engine.timeline));
        let added: Vec<Frames> = ours.iter().filter(|span| !theirs.contains(span)).copied().collect();
        let missing: Vec<Frames> = theirs.iter().filter(|span| !ours.contains(span)).copied().collect();
        println!(
            "{name}: {} windows against {} in the reference; only here {added:?}; only in the reference {missing:?}",
            ours.len(),
            theirs.len()
        );
        if ours != theirs {
            diverging.push((name.clone(), ours.len(), theirs.len(), added.len(), missing.len()));
        }
        videos.push(name);
    }

    assert_eq!(videos.len(), RECORDED_VIDEOS, "{videos:?}");
    let diverging: Vec<(&str, usize, usize, usize, usize)> = diverging
        .iter()
        .map(|(name, ours, theirs, added, missing)| (name.as_str(), *ours, *theirs, *added, *missing))
        .collect();
    assert_eq!(
        diverging,
        [
            ("video_093", 4, 6, 2, 4),
            ("video_094", 13, 14, 1, 2),
            ("video_095", 67, 78, 10, 21),
            ("video_096", 123, 136, 12, 25),
            ("video_097", 134, 149, 11, 26),
            ("video_098", 123, 131, 7, 15),
            ("video_099", 110, 117, 7, 14),
            ("video_100", 93, 98, 5, 10),
            ("video_101", 31, 32, 1, 2),
            ("video_102", 35, 36, 1, 2),
            ("video_125", 44, 47, 3, 6),
        ]
    );
}
