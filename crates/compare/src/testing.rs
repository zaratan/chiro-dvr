use reference::recording::Recording;
use types::detection::FrameDetections;

pub(crate) fn nothing_detected(recording: &Recording) -> FrameDetections {
    let mut empty = FrameDetections::default();
    for _ in 0..recording.frames.detections().frame_count() {
        empty.push([]);
    }
    empty
}
