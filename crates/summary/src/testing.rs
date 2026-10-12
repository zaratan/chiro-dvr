use types::detection::Detection;
use types::frame::{FrameIndex, Pts};
use types::rate::FrameRate;
use types::rational::Rational;
use types::timeline::Timeline;
use types::track::Track;

use crate::geometry::{Spot, Trajectory};

pub(crate) fn detection(frame: usize, [x, y]: Spot) -> Detection {
    Detection {
        frame: FrameIndex::new(frame),
        x,
        y,
        left: x - 2.0,
        top: y - 2.0,
        width: 4.0,
        height: 4.0,
        area: 16.0,
        amplitude: 30.0,
    }
}

pub(crate) fn track(id: usize, first_frame: usize, points: &[Spot]) -> Track {
    let detections = points
        .iter()
        .enumerate()
        .map(|(step, &point)| detection(first_frame + step, point))
        .collect();
    Track::new(id, detections).unwrap()
}

pub(crate) fn path(points: &[Spot]) -> Trajectory {
    Trajectory::new(points.to_vec())
}

pub(crate) fn horizontal(y: f64) -> Trajectory {
    path(&[[0.0, y], [100.0, y]])
}

pub(crate) fn timeline_at(rate: Rational, frames: usize) -> Timeline {
    let time_base = Rational::new(rate.den(), rate.num()).unwrap();
    let pts = (0..frames).map(|frame| Some(Pts::new(frame as i64))).collect();
    Timeline::new(pts, time_base, FrameRate::new(rate).unwrap()).unwrap()
}

pub(crate) fn timeline(frames: usize) -> Timeline {
    timeline_at(Rational::new(10, 1).unwrap(), frames)
}

#[derive(Debug)]
pub(crate) struct DebugLabels;

impl crate::header::Labels for DebugLabels {
    fn line(&self, line: &crate::header::HeaderLine) -> String {
        match line {
            crate::header::HeaderLine::Name(name) => name.clone(),
            other => format!("{other:?}"),
        }
    }
}
