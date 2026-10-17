use std::num::NonZeroU32;

use types::detection::{Detection, FrameDetections, Measures};
use types::frame::FrameIndex;
use types::size::SourceSize;
use types::span::Span;

use crate::exclusion::Exclusion;

pub(crate) fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

pub(crate) fn crowd(counts: &[usize]) -> FrameDetections {
    let mut frames = FrameDetections::default();
    for &count in counts {
        frames.push((0..count).map(|blob| Measures {
            x: 10.0 * blob as f64,
            y: 10.0,
            left: 10.0 * blob as f64 - 1.0,
            top: 9.0,
            width: 3.0,
            height: 3.0,
            area: 9.0,
            amplitude: 80.0,
        }));
    }
    frames
}

pub(crate) fn detection(frame: usize, x: f64, y: f64) -> Detection {
    Detection {
        frame: FrameIndex::new(frame),
        x,
        y,
        left: x.floor() - 1.0,
        top: y.floor() - 1.0,
        width: 3.0,
        height: 3.0,
        area: 9.0,
        amplitude: 80.0,
    }
}

pub(crate) fn line(start_frame: usize, frames: usize, start: (f64, f64), step: (f64, f64)) -> Vec<Detection> {
    (0..frames)
        .map(|offset| {
            let along = offset as f64;
            detection(start_frame + offset, start.0 + step.0 * along, start.1 + step.1 * along)
        })
        .collect()
}

pub(crate) fn frames_of(detections: &[Detection], frame_count: usize) -> FrameDetections {
    let mut placed = vec![Vec::new(); frame_count];
    for detection in detections {
        placed[detection.frame.get()].push(Measures::of(detection));
    }
    let mut frames = FrameDetections::default();
    for found in placed {
        frames.push(found);
    }
    frames
}

pub(crate) fn excluded(detections: FrameDetections, damaged: Vec<Span>) -> Exclusion {
    Exclusion {
        detections,
        damaged,
        unstable: Vec::new(),
        reported_frames: 0,
        ignored_frame_count: 0,
        damaged_frame_count: 0,
    }
}

pub(crate) fn size() -> SourceSize {
    SourceSize::new(NonZeroU32::new(1440).unwrap(), NonZeroU32::new(1080).unwrap())
}
