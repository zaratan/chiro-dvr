use std::io::{self, Write};
use std::path::Path;
use std::sync::Mutex;
use std::thread;

use engine::config::DetectConfig;
use engine::detection::detect;
use media::decode::Threads;
use reference::location::{recorded_video, recording_directories};
use reference::recording::Recording;
use reference::track_matching::{Discrepancy, match_tracks};
use tracking::exclusion::exclude;
use tracking::stability::StabilityConfig;
use tracking::track::{TrackConfig, track_detections};
use types::rate::FrameRate;

const VIDEOS_AT_ONCE: usize = 6;

struct Tracked {
    video: String,
    reference: usize,
    paired: usize,
    agreeing: usize,
    lost: usize,
    added: usize,
    discrepancies: Vec<Discrepancy>,
}

#[cfg(test)]
fn tracked(directory: &Path) -> Tracked {
    let recording = Recording::read(directory).unwrap();
    let summary = &recording.summary;
    let rate = FrameRate::new(summary.stream.avg_frame_rate).unwrap();
    let path = recorded_video(&summary.video.name).unwrap();
    let detected = detect(&path, Threads::Automatic, &DetectConfig::default()).unwrap();
    let exclusion = exclude(
        detected.detections(),
        summary.damaged_spans.clone(),
        summary.counts.damaged_frames_reported,
        rate,
        StabilityConfig::default(),
    )
    .unwrap();
    let tracks = track_detections(&exclusion.detections, TrackConfig::default()).unwrap();
    let matching = match_tracks(&recording.tracks, &tracks);
    Tracked {
        video: summary.video.name.clone(),
        reference: recording.tracks.len(),
        paired: matching.pairs.len(),
        agreeing: matching.agreeing(),
        lost: matching.reference_only.len(),
        added: matching.candidate_only.len(),
        discrepancies: matching.discrepancies(&summary.video.name, rate),
    }
}

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_detected_then_tracked_lose_three_of_the_966_reference_tracks() {
    let directories = Mutex::new(recording_directories().unwrap());
    let found = Mutex::new(Vec::new());
    thread::scope(|scope| {
        for _ in 0..VIDEOS_AT_ONCE {
            scope.spawn(|| {
                loop {
                    let next = directories.lock().unwrap().pop();
                    let Some(directory) = next else {
                        break;
                    };
                    let video = tracked(&directory);
                    found.lock().unwrap().push(video);
                }
            });
        }
    });
    let mut found = found.into_inner().unwrap();
    found.sort_by(|a, b| a.video.cmp(&b.video));
    let mut out = io::stdout().lock();
    for video in &found {
        writeln!(
            out,
            "{}: reference {}, paired {} (agreeing {}), lost {}, added {}",
            video.video, video.reference, video.paired, video.agreeing, video.lost, video.added
        )
        .unwrap();
    }
    for discrepancy in found.iter().flat_map(|video| &video.discrepancies) {
        writeln!(out, "{discrepancy:?}").unwrap();
    }
    let total = |count: fn(&Tracked) -> usize| found.iter().map(count).sum::<usize>();
    assert_eq!(found.len(), 26);
    assert_eq!(
        [
            total(|video| video.reference),
            total(|video| video.paired),
            total(|video| video.lost),
            total(|video| video.added)
        ],
        [966, 963, 3, 9]
    );
}
