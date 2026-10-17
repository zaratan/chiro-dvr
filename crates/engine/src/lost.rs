use types::frame::{FrameIndex, Pts};
use types::losses::EdgeLosses;

#[cfg(test)]
mod tests;

struct Interval {
    last: usize,
    frames: usize,
    ticks: f64,
}

impl Interval {
    fn frames_by(&self, step: f64) -> f64 {
        (self.ticks / step).round()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Losses {
    pub before_first: usize,
    pub at_jumps: usize,
    pub after_last: usize,
}

impl Losses {
    pub fn total(self) -> usize {
        self.before_first
            .saturating_add(self.at_jumps)
            .saturating_add(self.after_last)
    }

    pub fn edges(self) -> EdgeLosses {
        EdgeLosses {
            before_first: self.before_first,
            after_last: self.after_last,
        }
    }
}

pub(crate) fn losses(pts: &[Option<Pts>], start: Option<Pts>, after_last: usize) -> Losses {
    let intervals = intervals(pts);
    let Some(step) = step(&intervals) else {
        return Losses {
            after_last,
            ..Losses::default()
        };
    };
    Losses {
        before_first: before_first(pts, start, step),
        at_jumps: at_jumps(&intervals, step),
        after_last,
    }
}

fn before_first(pts: &[Option<Pts>], start: Option<Pts>, step: f64) -> usize {
    let first = pts
        .iter()
        .enumerate()
        .find_map(|(index, pts)| pts.map(|pts| (index, pts)));
    let (Some(start), Some((index, first))) = (start, first) else {
        return 0;
    };
    let frames = (ticks(i128::from(first.get()) - i128::from(start.get())) / step).round() - count(index);
    if frames > 0.0 { whole_frames(frames) } else { 0 }
}

fn at_jumps(intervals: &[Interval], step: f64) -> usize {
    intervals
        .iter()
        .map(|interval| interval.frames_by(step) - count(interval.frames))
        .filter(|lost| *lost > 0.0)
        .map(whole_frames)
        .fold(0, usize::saturating_add)
}

pub(crate) fn frames_after_jumps(pts: &[Option<Pts>]) -> Vec<FrameIndex> {
    let intervals = intervals(pts);
    let Some(step) = step(&intervals) else {
        return Vec::new();
    };
    intervals
        .iter()
        .filter(|interval| interval.frames_by(step) > count(interval.frames))
        .map(|interval| FrameIndex::new(interval.last))
        .collect()
}

fn intervals(pts: &[Option<Pts>]) -> Vec<Interval> {
    let known: Vec<(usize, i64)> = pts
        .iter()
        .enumerate()
        .filter_map(|(index, pts)| pts.map(|pts| (index, pts.get())))
        .collect();
    known
        .iter()
        .zip(known.iter().skip(1))
        .map(|((first, from), (last, to))| Interval {
            last: *last,
            frames: last - first,
            ticks: ticks(i128::from(*to) - i128::from(*from)),
        })
        .collect()
}

fn step(intervals: &[Interval]) -> Option<f64> {
    let median = median_step(intervals)?;
    let regular = intervals
        .iter()
        .filter(|interval| whole_frames(interval.frames_by(median)) == interval.frames);
    let (frames, ticks) = regular.fold((0, 0.0), |(frames, ticks), interval| {
        (frames + interval.frames, ticks + interval.ticks)
    });
    Some(ticks / count(frames))
        .filter(|step| step.is_finite())
        .or(Some(median))
}

fn median_step(intervals: &[Interval]) -> Option<f64> {
    let mut steps: Vec<f64> = intervals
        .iter()
        .map(|interval| interval.ticks / count(interval.frames))
        .collect();
    if steps.is_empty() {
        return None;
    }
    let middle = steps.len() / 2;
    let (_, median, _) = steps.select_nth_unstable_by(middle, f64::total_cmp);
    Some(*median).filter(|step| *step > 0.0)
}

fn count(frames: usize) -> f64 {
    frames as f64
}

fn ticks(ticks: i128) -> f64 {
    ticks as f64
}

fn whole_frames(frames: f64) -> usize {
    frames.round() as usize
}
