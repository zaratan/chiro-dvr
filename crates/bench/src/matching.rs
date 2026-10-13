use types::detection::Detection;
use types::frame::FrameIndex;
use types::track::Track;

use crate::config::MatchConfig;
use crate::synthetic::Point;
use crate::synthetic::injection::Injected;

#[cfg(test)]
mod tests;

pub(crate) fn near(detection: &Detection, at: Point, radius: f64) -> bool {
    (detection.x - at.x).hypot(detection.y - at.y) <= radius
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ByFrame<T>(Vec<Vec<T>>);

impl<T> Default for ByFrame<T> {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl<T> ByFrame<T> {
    fn add(&mut self, frame: FrameIndex, item: T) {
        let index = frame.get();
        if self.0.len() <= index {
            self.0.resize_with(index + 1, Vec::new);
        }
        self.0[index].push(item);
    }

    pub(crate) fn at(&self, frame: FrameIndex) -> &[T] {
        self.0.get(frame.get()).map_or(&[], Vec::as_slice)
    }
}

pub(crate) fn truth(injected: &[Injected]) -> ByFrame<(usize, Point)> {
    let mut truth = ByFrame::default();
    for (index, placed) in injected.iter().enumerate() {
        for observation in &placed.observations {
            truth.add(observation.frame, (index, observation.at));
        }
    }
    truth
}

pub(crate) fn reference_points(tracks: &[Track]) -> ByFrame<Point> {
    let mut points = ByFrame::default();
    for detection in tracks.iter().flat_map(Track::points) {
        points.add(
            detection.frame,
            Point {
                x: detection.x,
                y: detection.y,
            },
        );
    }
    points
}

pub(crate) fn assign_tracks(
    tracks: &[Track],
    truth: &ByFrame<(usize, Point)>,
    matching: MatchConfig,
) -> Vec<Option<usize>> {
    tracks
        .iter()
        .map(|track| assigned_target(track, truth, matching))
        .collect()
}

fn assigned_target(track: &Track, truth: &ByFrame<(usize, Point)>, matching: MatchConfig) -> Option<usize> {
    let mut counts: Vec<(usize, usize)> = Vec::new();
    for detection in track.points() {
        for &(target, at) in truth.at(detection.frame) {
            if near(detection, at, matching.radius()) {
                match counts.iter_mut().find(|(counted, _)| *counted == target) {
                    Some((_, count)) => *count += 1,
                    None => counts.push((target, 1)),
                }
            }
        }
    }
    let mut best: Option<(usize, usize)> = None;
    for &(target, count) in &counts {
        if best.is_none_or(|(_, most)| count > most) {
            best = Some((target, count));
        }
    }
    best.filter(|&(_, count)| pure(count, track, matching))
        .map(|(target, _)| target)
}

pub(crate) fn matches_reference(track: &Track, points: &ByFrame<Point>, matching: MatchConfig) -> bool {
    let close = track
        .points()
        .iter()
        .filter(|detection| {
            points
                .at(detection.frame)
                .iter()
                .any(|&point| near(detection, point, matching.radius()))
        })
        .count();
    pure(close, track, matching)
}

fn pure(close: usize, track: &Track, matching: MatchConfig) -> bool {
    close as f64 / track.points().len() as f64 >= matching.purity()
}
