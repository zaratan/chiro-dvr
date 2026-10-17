use std::num::{NonZeroU32, NonZeroUsize};
use std::path::{Path, PathBuf};

use engine::settings::DetectSettings;
use types::detection::{Detection, FrameDetections, Measures, Strength};
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::rational::Rational;
use types::size::SourceSize;
use types::span::Span;

use crate::noisy;
use crate::run::{Provenance, Run, Video};
use crate::synthetic::injection::{Injected, Observation};
use crate::synthetic::{Class, Motion, Point, Scene, Target};

pub(crate) const STEP: f64 = 18.0;

pub(crate) fn size() -> SourceSize {
    SourceSize::new(NonZeroU32::new(480).unwrap(), NonZeroU32::new(360).unwrap())
}

pub(crate) fn rate() -> FrameRate {
    FrameRate::new(Rational::new(30, 1).unwrap()).unwrap()
}

pub(crate) fn scene() -> Scene {
    Scene::new(size(), rate())
}

pub(crate) fn detection(frame: usize, x: f64, y: f64) -> Detection {
    Detection {
        frame: FrameIndex::new(frame),
        x,
        y,
        left: x - 2.0,
        top: y - 2.0,
        width: 5.0,
        height: 5.0,
        area: 20.0,
        amplitude: -30.0,
        strength: Strength::Strong,
    }
}

pub(crate) fn along(id: usize, start: usize, frames: usize, y: f64) -> Target {
    let positions = (0..frames)
        .map(|offset| Point {
            x: 90.0 + STEP * offset as f64,
            y,
        })
        .collect();
    let class = Class::new(-40.0, 3.0, Motion::Pass).unwrap();
    Target::new(id, FrameIndex::new(start), positions, vec![-40.0; frames], class).unwrap()
}

pub(crate) fn observed(target: Target, effective: f64) -> Injected {
    let observations = (target.start().get()..=target.end().get())
        .map(|frame| Observation {
            frame: FrameIndex::new(frame),
            at: target.position(FrameIndex::new(frame)).unwrap(),
            effective,
        })
        .collect();
    Injected { target, observations }
}

pub(crate) fn on(target: &Target) -> Vec<Detection> {
    (target.start().get()..=target.end().get())
        .map(|frame| {
            let at = target.position(FrameIndex::new(frame)).unwrap();
            detection(frame, at.x, at.y)
        })
        .collect()
}

pub(crate) fn noisy_video(path: &Path) -> PathBuf {
    noisy::write_video(path).unwrap();
    path.to_path_buf()
}

pub(crate) fn slow_target(id: usize, start: usize, frames: usize) -> Target {
    noisy::slow_target(id, start, frames).unwrap()
}

pub(crate) const RUN_FRAMES: usize = 100;

pub(crate) fn frames(detections: impl IntoIterator<Item = Detection>) -> FrameDetections {
    let mut placed = vec![Vec::new(); RUN_FRAMES];
    for detection in detections {
        placed[detection.frame.get()].push(Measures::of(&detection));
    }
    let mut frames = FrameDetections::default();
    for found in placed {
        frames.push(found);
    }
    frames
}

pub(crate) fn run(
    targets: Vec<Injected>,
    injected: Vec<Detection>,
    reference: Vec<Detection>,
    damaged: Vec<Span>,
) -> Run {
    Run {
        video: Video {
            path: PathBuf::from("video.mp4"),
            size: size(),
            rate: rate(),
            damaged,
            reported_frames: 0,
            half_window: NonZeroUsize::new(15).unwrap(),
        },
        detect: DetectSettings::default(),
        reference: frames(reference),
        injected: frames(injected),
        targets,
        provenance: Provenance::Replayed(PathBuf::from("bats.jsonl.gz")),
    }
}
