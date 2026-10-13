use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use engine::config::DetectConfig;
use engine::detection::detect;
use media::decode::{Decoder, Threads};
use media::fingerprint;
use media::frame::{ColorRange, YuvFrame};
use reference::location::{recorded_video, recording_directories};
use reference::recording::Recording;
use render::config::RenderConfig;
use render::source::{Error, Origin, Source};
use render::windows::{Group, Window, clip_windows, groups};
use tracking::exclusion::exclude;
use tracking::stability::StabilityConfig;
use tracking::track::{TrackConfig, track_detections};
use types::span::Span;
use types::timeline::Timeline;
use types::track::Track;

const DAMAGED: [(&str, usize); 3] = [("video_103", 0), ("video_123", 0), ("video_124", 0)];
type Frames = (usize, usize);
const DIVERGING: [(&str, &[Frames], &[Frames]); 2] = [
    (
        "video_097",
        &[(574, 748), (2942, 3068), (4827, 4943), (4861, 5018)],
        &[(567, 748), (2946, 3068), (4827, 5018)],
    ),
    ("video_125", &[], &[]),
];

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
    let detected = detect(&video, Threads::Automatic, &DetectConfig::default()).unwrap();
    let reading = detected.reading();
    let rate = reading.video().frame_rate();
    let pts = reading.frames().iter().map(|frame| frame.pts()).collect();
    let timeline = Timeline::new(pts, reading.video().time_base(), rate).unwrap();
    let damage = detected.damage();
    let exclusion = exclude(
        detected.detections(),
        damage.spans().to_vec(),
        damage.reported_frames(),
        rate,
        StabilityConfig::default(),
    );
    let tracks = track_detections(&exclusion.detections, TrackConfig::default()).unwrap();
    let source = Source::new(&video, &timeline, &reading.key_frames(), damage.error_frames()).unwrap();
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
fn the_windows_differ_from_the_reference_only_at_the_known_divergences() {
    let directories = recording_directories().unwrap();
    for (name, expected_here, expected_there) in DIVERGING {
        let directory = directories.iter().find(|directory| directory.ends_with(name)).unwrap();
        let recording = Recording::read(directory).unwrap();
        let engine = through_the_engine(name);
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
        assert_eq!(
            (added.as_slice(), missing.as_slice()),
            (expected_here, expected_there),
            "{name}"
        );
    }
}
