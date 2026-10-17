use std::path::Path;

use engine::input::Input;
use media::decode::Threads;
use reference::frames::Frames;
use reference::location::{Reference, committed};
use reference::recording::{Recording, SUMMARY_FILE};
use reference::video::VideoSummary;
use types::detection::FrameDetections;

use crate::case::EXTRACT;
use crate::damaged::reported_by;
use crate::frames::Pass;
use crate::provenance::Build;

pub(crate) const SPOT: &str = "spot.mp4";
pub(crate) const WORK_WIDTH: u32 = 240;

pub(crate) fn extract() -> Recording {
    Recording::read(&committed(Reference::V020, EXTRACT)).unwrap()
}

pub(crate) fn pass_finding(recording: &Recording, detections: FrameDetections) -> Pass {
    Pass {
        frames: Frames::new(recording.frames.entries().to_vec(), detections).unwrap(),
        reports: reported_by(&recording.summary),
    }
}

pub(crate) fn same_pass(recording: &Recording) -> Pass {
    pass_finding(recording, recording.frames.detections().clone())
}

pub(crate) fn nothing_detected(recording: &Recording) -> FrameDetections {
    let mut empty = FrameDetections::default();
    for _ in 0..recording.frames.detections().frame_count() {
        empty.push([]);
    }
    empty
}

pub(crate) fn frames_without_detections(recording: &Recording) -> Frames {
    Frames::new(recording.frames.entries().to_vec(), nothing_detected(recording)).unwrap()
}

pub(crate) fn build() -> Build {
    Build::clean("v0.2.0-1-g28d5526", "28d5526d0b2c", "rustc 1.90.0").unwrap()
}

pub(crate) fn probed(path: &Path) -> VideoSummary {
    let mut summary =
        VideoSummary::read(&committed(Reference::V011, "video_092_original_3m24-4m05").join(SUMMARY_FILE)).unwrap();
    let video = Input::open(path, Threads::Automatic).unwrap().video().clone();
    summary.video.name = path.to_string_lossy().into_owned();
    summary.stream.width = video.size().width().get();
    summary.stream.height = video.size().height().get();
    summary.stream.avg_frame_rate = video.frame_rate().get();
    summary.stream.time_base = video.time_base();
    summary.recorded["config"]["DetectConfig"]["work_width"] = WORK_WIDTH.into();
    summary
}
