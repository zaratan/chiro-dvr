use std::num::NonZeroU32;

use crate::frame::FrameIndex;
use crate::rational::{DivisionError, Rational};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FrameRate(Rational);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("the frame rate {0} is not positive")]
pub struct Error(Rational);

impl FrameRate {
    pub fn new(rate: Rational) -> Result<Self, Error> {
        if rate.num() <= 0 {
            return Err(Error(rate));
        }
        Ok(Self(rate))
    }

    pub fn get(self) -> Rational {
        self.0
    }

    pub fn slowed(self, factor: NonZeroU32) -> Result<Self, DivisionError> {
        self.0.divided_by(factor).map(Self)
    }

    pub fn seconds(self, frame: FrameIndex) -> f64 {
        frame.get() as f64 * f64::from(self.0.den()) / f64::from(self.0.num())
    }
}
