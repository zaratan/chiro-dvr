use types::detection::{Detection, FrameDetections};
use types::frame::FrameIndex;

#[cfg(test)]
mod tests;

pub const DETECTION_DISTANCE_PX: f64 = 3.0;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct DetectionMatching {
    pub pairs: Vec<(Detection, Detection)>,
    pub reference_only: Vec<Detection>,
    pub candidate_only: Vec<Detection>,
}

impl DetectionMatching {
    pub fn is_complete(&self) -> bool {
        self.reference_only.is_empty() && self.candidate_only.is_empty()
    }
}

pub fn match_detections(reference: &FrameDetections, candidate: &FrameDetections) -> DetectionMatching {
    let mut matching = DetectionMatching::default();
    for frame in (0..reference.frame_count().max(candidate.frame_count())).map(FrameIndex::new) {
        let reference = reference.get(frame).unwrap_or_default();
        let candidate = candidate.get(frame).unwrap_or_default();
        match_frame(reference, candidate, &mut matching);
    }
    matching
}

fn match_frame(reference: &[Detection], candidate: &[Detection], matching: &mut DetectionMatching) {
    let mut by_x: Vec<usize> = (0..candidate.len()).filter(|&c| candidate[c].x.is_finite()).collect();
    by_x.sort_by(|&a, &b| candidate[a].x.total_cmp(&candidate[b].x));
    let mut close = Vec::new();
    for (r, kept) in reference.iter().enumerate() {
        let from = by_x.partition_point(|&c| candidate[c].x <= kept.x - DETECTION_DISTANCE_PX);
        close.extend(
            by_x[from..]
                .iter()
                .take_while(|&&c| candidate[c].x < kept.x + DETECTION_DISTANCE_PX)
                .map(|&c| (kept.distance(&candidate[c]), r, c))
                .filter(|&(distance, _, _)| distance < DETECTION_DISTANCE_PX),
        );
    }
    close.sort_by(|a, b| a.0.total_cmp(&b.0).then((a.1, a.2).cmp(&(b.1, b.2))));
    let mut reference_taken = vec![false; reference.len()];
    let mut candidate_taken = vec![false; candidate.len()];
    for (_, r, c) in close {
        if !reference_taken[r] && !candidate_taken[c] {
            reference_taken[r] = true;
            candidate_taken[c] = true;
            matching.pairs.push((reference[r], candidate[c]));
        }
    }
    matching.reference_only.extend(untaken(reference, &reference_taken));
    matching.candidate_only.extend(untaken(candidate, &candidate_taken));
}

fn untaken<'a>(detections: &'a [Detection], taken: &'a [bool]) -> impl Iterator<Item = Detection> + 'a {
    detections
        .iter()
        .zip(taken)
        .filter(|&(_, &taken)| !taken)
        .map(|(detection, _)| *detection)
}
