use crate::frame::{FrameIndex, Pts};
use crate::rate::FrameRate;
use crate::rational::Rational;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timeline {
    pts: Vec<Option<Pts>>,
    time_base: Rational,
    rate: FrameRate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the time base {0} is not positive")]
pub struct Error(Rational);

impl Timeline {
    pub fn new(pts: Vec<Option<Pts>>, time_base: Rational, rate: FrameRate) -> Result<Self, Error> {
        if time_base.num() <= 0 {
            return Err(Error(time_base));
        }
        Ok(Self { pts, time_base, rate })
    }

    pub fn frame_count(&self) -> usize {
        self.pts.len()
    }

    pub fn time_base(&self) -> Rational {
        self.time_base
    }

    pub fn rate(&self) -> FrameRate {
        self.rate
    }

    pub fn pts(&self, frame: FrameIndex) -> Option<Pts> {
        self.pts.get(frame.get()).copied().flatten()
    }

    pub fn seconds_by_pts(&self, frame: FrameIndex) -> Option<f64> {
        let ticks = i128::from(self.pts(frame)?.get()) * i128::from(self.time_base.num());
        Some(ticks as f64 / f64::from(self.time_base.den()))
    }

    pub fn seconds_by_rate(&self, frame: FrameIndex) -> f64 {
        self.rate.seconds(frame)
    }
}
