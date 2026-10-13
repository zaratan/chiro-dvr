use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::size::SourceSize;

pub mod hunting;
pub mod injection;
pub mod sampling;
pub mod trajectory;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Motion {
    Pass,
    Hunt,
    Circle,
}

impl Motion {
    pub const ALL: [Self; 3] = [Self::Pass, Self::Hunt, Self::Circle];

    pub fn name(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Hunt => "hunt",
            Self::Circle => "circle",
        }
    }
}

impl fmt::Display for Motion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown motion {0:?}, expected pass, hunt or circle")]
pub struct UnknownMotion(pub String);

impl FromStr for Motion {
    type Err = UnknownMotion;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|motion| motion.name() == name)
            .ok_or_else(|| UnknownMotion(name.to_owned()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Class {
    amplitude: f64,
    sigma: f64,
    motion: Motion,
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("the amplitude of a target must be a finite number, not {0}")]
    Amplitude(f64),
    #[error("the sigma of a target must be a finite number > 0, not {0}")]
    Sigma(f64),
    #[error("target {id} has {positions} positions and {amplitudes} amplitudes, one of each per frame was expected")]
    Lengths {
        id: usize,
        positions: usize,
        amplitudes: usize,
    },
    #[error("target {id} has no position")]
    Empty { id: usize },
    #[error("target {id} ends beyond the last frame that can be numbered")]
    TooLong { id: usize },
}

impl Class {
    pub fn new(amplitude: f64, sigma: f64, motion: Motion) -> Result<Self, Error> {
        if !amplitude.is_finite() {
            return Err(Error::Amplitude(amplitude));
        }
        if !(sigma.is_finite() && sigma > 0.0) {
            return Err(Error::Sigma(sigma));
        }
        Ok(Self {
            amplitude,
            sigma,
            motion,
        })
    }

    pub fn amplitude(self) -> f64 {
        self.amplitude
    }

    pub fn sigma(self) -> f64 {
        self.sigma
    }

    pub fn motion(self) -> Motion {
        self.motion
    }

    pub fn order(self, other: Self) -> Ordering {
        self.motion
            .cmp(&other.motion)
            .then(self.amplitude.total_cmp(&other.amplitude))
            .then(self.sigma.total_cmp(&other.sigma))
    }

    pub(crate) fn margin(self) -> f64 {
        3.0 * self.sigma + 2.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn distance(self, other: Self) -> f64 {
        (self.x - other.x).hypot(self.y - other.y)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scene {
    size: SourceSize,
    rate: FrameRate,
}

impl Scene {
    pub fn new(size: SourceSize, rate: FrameRate) -> Self {
        Self { size, rate }
    }

    pub fn width(self) -> f64 {
        f64::from(self.size.width().get())
    }

    pub fn height(self) -> f64 {
        f64::from(self.size.height().get())
    }

    pub fn fps(self) -> f64 {
        self.rate.get().to_f64()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Target {
    id: usize,
    start: FrameIndex,
    positions: Vec<Point>,
    amplitudes: Vec<f64>,
    class: Class,
}

impl Target {
    pub fn new(
        id: usize,
        start: FrameIndex,
        positions: Vec<Point>,
        amplitudes: Vec<f64>,
        class: Class,
    ) -> Result<Self, Error> {
        if positions.len() != amplitudes.len() {
            return Err(Error::Lengths {
                id,
                positions: positions.len(),
                amplitudes: amplitudes.len(),
            });
        }
        if positions.is_empty() {
            return Err(Error::Empty { id });
        }
        if start.get().checked_add(positions.len()).is_none() {
            return Err(Error::TooLong { id });
        }
        Ok(Self {
            id,
            start,
            positions,
            amplitudes,
            class,
        })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn start(&self) -> FrameIndex {
        self.start
    }

    pub fn end(&self) -> FrameIndex {
        FrameIndex::new(self.start.get() + self.positions.len() - 1)
    }

    pub fn class(&self) -> Class {
        self.class
    }

    pub fn positions(&self) -> &[Point] {
        &self.positions
    }

    pub fn amplitudes(&self) -> &[f64] {
        &self.amplitudes
    }

    pub fn position(&self, frame: FrameIndex) -> Option<Point> {
        self.offset(frame).map(|offset| self.positions[offset])
    }

    pub fn amplitude_at(&self, frame: FrameIndex) -> Option<f64> {
        self.offset(frame).map(|offset| self.amplitudes[offset])
    }

    pub(crate) fn starting_at(self, start: FrameIndex) -> Self {
        Self { start, ..self }
    }

    fn offset(&self, frame: FrameIndex) -> Option<usize> {
        frame
            .get()
            .checked_sub(self.start.get())
            .filter(|&offset| offset < self.positions.len())
    }
}
