use types::detection::{Detection, FrameDetections};
use types::frame::FrameIndex;
use types::span::Span;

pub(crate) fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

pub(crate) fn crowd(counts: &[usize]) -> FrameDetections {
    let frames = counts
        .iter()
        .enumerate()
        .map(|(frame, &count)| {
            (0..count)
                .map(|blob| Detection {
                    frame: FrameIndex::new(frame),
                    x: 10.0 * blob as f64,
                    y: 10.0,
                    left: 10.0 * blob as f64 - 1.0,
                    top: 9.0,
                    width: 3.0,
                    height: 3.0,
                    area: 9.0,
                    amplitude: 80.0,
                })
                .collect()
        })
        .collect();
    FrameDetections::new(frames).unwrap()
}
