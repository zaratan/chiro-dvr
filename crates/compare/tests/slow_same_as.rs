use std::fs;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use compare::case::EXTRACT;
use compare::comparison::frames_path;
use compare::frames::frames_of;
use compare::sameness::Sameness;
use engine::config::DetectConfig;
use engine::detection::{Detected, detect};
use engine::settings::DetectSettings;
use media::decode::{ActiveThreads, Threads};
use reference::fixture::fixture;
use reference::frames::Frames;
use reference::location::{Reference, committed};
use reference::recording::Recording;
use tempfile::TempDir;
use types::detection::{FrameDetections, Measures};
use types::frame::FrameIndex;
use types::span::Span;

const PARTS: usize = 20;
const WIPED_PART: usize = 10;
const SHIFT_PX: f64 = 0.5;

struct Runs {
    threads: ActiveThreads,
    spans: Vec<Span>,
    sameness: Sameness,
    moved: FrameIndex,
    with_moved: Sameness,
}

fn video() -> String {
    format!("{EXTRACT}.mp4")
}

#[cfg(test)]
fn damaged_extract(folder: &Path) -> PathBuf {
    let path = folder.join(video());
    fs::copy(fixture(&video()).unwrap(), &path).unwrap();
    media::payload::wipe(&path, WIPED_PART..WIPED_PART + 1, PARTS).unwrap();
    path
}

#[cfg(test)]
fn detected(path: &Path, threads: Threads) -> Detected {
    let recording = Recording::read(&committed(Reference::V011, EXTRACT)).unwrap();
    let settings: DetectSettings = recording.summary.detect_settings().unwrap();
    detect(path, threads, &DetectConfig::try_from(&settings).unwrap()).unwrap()
}

#[cfg(test)]
fn written_and_read_back(frames: &Frames, folder: &Path) -> Frames {
    let path = frames_path(folder, &video());
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    frames.write(&path).unwrap();
    Frames::read(&path).unwrap()
}

fn first_outside(frames: &Frames, spans: &[Span]) -> Option<FrameIndex> {
    frames
        .detections()
        .iter()
        .find(|(frame, found)| !found.is_empty() && !spans.iter().any(|span| span.contains(*frame)))
        .map(|(frame, _)| frame)
}

fn moved_at(frames: &Frames, moved: FrameIndex) -> Option<Frames> {
    let mut detections = FrameDetections::default();
    for (frame, found) in frames.detections().iter() {
        detections.push(found.iter().enumerate().map(|(rank, detection)| Measures {
            x: if frame == moved && rank == 0 {
                detection.x + SHIFT_PX
            } else {
                detection.x
            },
            ..Measures::of(detection)
        }));
    }
    Frames::new(frames.entries().to_vec(), detections).ok()
}

#[cfg(test)]
fn runs() -> &'static Runs {
    static RUNS: OnceLock<Runs> = OnceLock::new();
    RUNS.get_or_init(|| {
        let folder = TempDir::new_in(env!("CARGO_TARGET_TMPDIR")).unwrap();
        let cut = damaged_extract(folder.path());
        let single = Threads::Exactly(NonZeroUsize::MIN);
        let previous = written_and_read_back(&frames_of(&detected(&cut, single)).unwrap(), folder.path());
        let automatic = detected(&cut, Threads::Automatic);
        let current = frames_of(&automatic).unwrap();
        let spans = automatic.damage().spans().to_vec();
        let moved = first_outside(&current, &spans).unwrap();
        Runs {
            threads: automatic.active_threads(),
            sameness: Sameness::of(&video(), &previous, &current, &spans),
            with_moved: Sameness::of(&video(), &previous, &moved_at(&current, moved).unwrap(), &spans),
            spans,
            moved,
        }
    })
}

#[test]
#[ignore = "slow"]
fn one_thread_and_automatic_threads_detect_the_same_outside_the_damaged_spans_of_the_cut_extract() {
    let runs = runs();

    assert!(matches!(runs.threads, ActiveThreads::Frame(_)), "{:?}", runs.threads);
    assert_ne!(runs.spans, [] as [Span; 0]);
    assert!(runs.sameness.holds(), "{:?}", runs.sameness);
}

#[test]
#[ignore = "slow"]
fn a_detection_moved_outside_the_damaged_spans_of_the_cut_extract_is_listed_with_its_video_and_frame() {
    let runs = runs();
    let mut line = Vec::new();

    runs.with_moved.write_line(&mut line).unwrap();

    assert!(!runs.with_moved.holds());
    assert!(String::from_utf8(line).unwrap().starts_with(&format!(
        "{}\tdiffers\toutside 1: {}\t",
        video(),
        runs.moved
    )));
}
