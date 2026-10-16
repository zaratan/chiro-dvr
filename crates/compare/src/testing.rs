use reference::frames::Frames;
use reference::recording::Recording;
use types::detection::FrameDetections;

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
