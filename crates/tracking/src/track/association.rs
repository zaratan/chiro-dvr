use types::detection::{Detection, Strength};
use types::frame::FrameIndex;

use crate::track::growing::Growing;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Age {
    Established,
    Newborn,
}

#[derive(Debug, Default)]
pub(crate) struct Association {
    candidates: Vec<(Age, f64, usize, usize)>,
    track_taken: Vec<bool>,
    detection_taken: Vec<bool>,
}

impl Association {
    pub(crate) fn assign<'a>(
        &'a mut self,
        active: &mut [Growing],
        detections: &'a [Detection],
        frame: FrameIndex,
        max_jump: f64,
    ) -> impl Iterator<Item = Detection> + 'a {
        self.track_taken.clear();
        self.track_taken.resize(active.len(), false);
        self.detection_taken.clear();
        self.detection_taken.resize(detections.len(), false);
        for strength in [Strength::Strong, Strength::Weak] {
            self.pair(active, detections, strength, frame, max_jump);
        }
        detections
            .iter()
            .zip(&self.detection_taken)
            .filter(|&(_, &taken)| !taken)
            .map(|(detection, _)| *detection)
    }

    fn pair(
        &mut self,
        active: &mut [Growing],
        detections: &[Detection],
        strength: Strength,
        frame: FrameIndex,
        max_jump: f64,
    ) {
        self.candidates.clear();
        for (track_index, track) in active.iter().enumerate() {
            if self.track_taken[track_index] {
                continue;
            }
            let (x, y) = track.predicted(frame);
            let age = if track.has_velocity() {
                Age::Established
            } else {
                Age::Newborn
            };
            for (detection_index, detection) in detections.iter().enumerate() {
                let distance = (detection.x - x).hypot(detection.y - y);
                if detection.strength == strength && distance <= max_jump {
                    self.candidates.push((age, distance, track_index, detection_index));
                }
            }
        }
        self.candidates.sort_unstable_by(|a, b| {
            a.0.cmp(&b.0)
                .then_with(|| a.1.total_cmp(&b.1))
                .then_with(|| (a.2, a.3).cmp(&(b.2, b.3)))
        });
        for &(_, _, track_index, detection_index) in &self.candidates {
            if !self.track_taken[track_index] && !self.detection_taken[detection_index] {
                self.track_taken[track_index] = true;
                self.detection_taken[detection_index] = true;
                active[track_index].push(detections[detection_index]);
            }
        }
    }
}
