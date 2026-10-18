use crate::error::Error;
use crate::setting::Setting;
use crate::track::positive;

#[cfg(test)]
mod tests;

const DEFAULT_MARGIN: f64 = 50.0;
const DEFAULT_TRAVEL: f64 = 1.0;

/// Farthest a track with a speed reaches from its predicted position: a margin plus a share of the way it was predicted to fly.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Reach {
    margin: f64,
    travel: f64,
}

impl Default for Reach {
    fn default() -> Self {
        Self {
            margin: DEFAULT_MARGIN,
            travel: DEFAULT_TRAVEL,
        }
    }
}

impl Reach {
    pub fn new(margin: f64, travel: f64) -> Result<Self, Error> {
        positive(margin, Setting::ReachMargin)?;
        if !(travel.is_finite() && travel >= 0.0) {
            return Err(Error::Setting {
                setting: Setting::ReachTravel,
                value: travel,
            });
        }
        Ok(Self { margin, travel })
    }

    pub fn margin(self) -> f64 {
        self.margin
    }

    pub fn travel(self) -> f64 {
        self.travel
    }

    pub(crate) fn farthest(self, way: f64) -> f64 {
        self.margin + self.travel * way
    }
}
