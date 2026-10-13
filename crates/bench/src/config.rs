use crate::synthetic::{self, Class, Motion};

#[cfg(test)]
mod tests;

pub const DEFAULT_AMPLITUDES: [f64; 3] = [-15.0, -30.0, -60.0];
pub const DEFAULT_SIGMAS: [f64; 3] = [1.5, 3.0, 5.0];
pub const DEFAULT_RADIUS: f64 = 24.0;
pub const DEFAULT_PURITY: f64 = 0.5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MatchConfig {
    radius: f64,
    purity: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("the match radius must be a finite number > 0, not {0}")]
    Radius(f64),
    #[error("the match purity must be in ]0, 1], not {0}")]
    Purity(f64),
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            radius: DEFAULT_RADIUS,
            purity: DEFAULT_PURITY,
        }
    }
}

impl MatchConfig {
    pub fn new(radius: f64, purity: f64) -> Result<Self, Error> {
        if !(radius.is_finite() && radius > 0.0) {
            return Err(Error::Radius(radius));
        }
        if !(purity > 0.0 && purity <= 1.0) {
            return Err(Error::Purity(purity));
        }
        Ok(Self { radius, purity })
    }

    pub fn radius(self) -> f64 {
        self.radius
    }

    pub fn purity(self) -> f64 {
        self.purity
    }
}

pub fn classes(motions: &[Motion], amplitudes: &[f64], sigmas: &[f64]) -> Result<Vec<Class>, synthetic::Error> {
    Motion::ALL
        .into_iter()
        .filter(|motion| motions.contains(motion))
        .flat_map(|motion| {
            amplitudes
                .iter()
                .flat_map(move |&amplitude| sigmas.iter().map(move |&sigma| Class::new(amplitude, sigma, motion)))
        })
        .collect()
}
