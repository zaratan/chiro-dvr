use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::track::Track;

#[cfg(test)]
mod tests;

pub const START_TOLERANCE_FRAMES: usize = 5;
pub const POINTS_TOLERANCE: usize = 3;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct TrackMatching<'a> {
    pub pairs: Vec<(&'a Track, &'a Track)>,
    pub reference_only: Vec<&'a Track>,
    pub candidate_only: Vec<&'a Track>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Discrepancy {
    pub video: String,
    pub frame: FrameIndex,
    pub seconds: f64,
    pub kind: DiscrepancyKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscrepancyKind {
    Missed {
        reference_id: usize,
    },
    Extra {
        candidate_id: usize,
    },
    Points {
        reference_id: usize,
        candidate_id: usize,
        reference_points: usize,
        candidate_points: usize,
    },
}

pub fn match_tracks<'a>(reference: &'a [Track], candidate: &'a [Track]) -> TrackMatching<'a> {
    let mut close: Vec<(usize, usize, usize, usize)> = reference
        .iter()
        .enumerate()
        .flat_map(|(r, kept)| {
            candidate
                .iter()
                .enumerate()
                .map(move |(c, found)| (start_gap(kept, found), points_gap(kept, found), r, c))
        })
        .filter(|&(start_gap, ..)| start_gap <= START_TOLERANCE_FRAMES)
        .collect();
    close.sort_unstable();
    let mut reference_taken = vec![false; reference.len()];
    let mut candidate_taken = vec![false; candidate.len()];
    let mut pairs = Vec::new();
    for (.., r, c) in close {
        if !reference_taken[r] && !candidate_taken[c] {
            reference_taken[r] = true;
            candidate_taken[c] = true;
            pairs.push((&reference[r], &candidate[c]));
        }
    }
    TrackMatching {
        pairs,
        reference_only: untaken(reference, &reference_taken),
        candidate_only: untaken(candidate, &candidate_taken),
    }
}

impl TrackMatching<'_> {
    pub fn agreeing(&self) -> usize {
        self.pairs
            .iter()
            .filter(|(kept, found)| points_gap(kept, found) <= POINTS_TOLERANCE)
            .count()
    }

    pub fn discrepancies(&self, video: &str, rate: FrameRate) -> Vec<Discrepancy> {
        let missed = self.reference_only.iter().map(|kept| {
            (
                kept.first().frame,
                DiscrepancyKind::Missed {
                    reference_id: kept.id(),
                },
            )
        });
        let extra = self.candidate_only.iter().map(|found| {
            (
                found.first().frame,
                DiscrepancyKind::Extra {
                    candidate_id: found.id(),
                },
            )
        });
        let points = self
            .pairs
            .iter()
            .filter(|(kept, found)| points_gap(kept, found) > POINTS_TOLERANCE)
            .map(|(kept, found)| {
                let kind = DiscrepancyKind::Points {
                    reference_id: kept.id(),
                    candidate_id: found.id(),
                    reference_points: kept.points().len(),
                    candidate_points: found.points().len(),
                };
                (kept.first().frame, kind)
            });
        let mut discrepancies: Vec<Discrepancy> = missed
            .chain(extra)
            .chain(points)
            .map(|(frame, kind)| Discrepancy {
                video: video.to_owned(),
                frame,
                seconds: rate.seconds(frame),
                kind,
            })
            .collect();
        discrepancies.sort_by_key(|discrepancy| discrepancy.frame);
        discrepancies
    }
}

fn start_gap(kept: &Track, found: &Track) -> usize {
    kept.first().frame.abs_diff(found.first().frame)
}

fn points_gap(kept: &Track, found: &Track) -> usize {
    kept.points().len().abs_diff(found.points().len())
}

fn untaken<'a>(tracks: &'a [Track], taken: &[bool]) -> Vec<&'a Track> {
    tracks
        .iter()
        .zip(taken)
        .filter(|&(_, &taken)| !taken)
        .map(|(track, _)| track)
        .collect()
}
