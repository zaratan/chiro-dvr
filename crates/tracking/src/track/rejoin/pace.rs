use types::detection::Detection;
use types::track::Track;

use crate::error::Error;
use crate::setting::Setting;
use crate::track::rejoin::{Role, head, tail, velocity};

#[cfg(test)]
mod tests;

const DEFAULT_JUNCTION: f64 = 0.25;
const DEFAULT_PIECE: f64 = 0.45;

/// Slowest speeds a rejoin accepts, as fractions of the speed of the kept track it continues.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RejoinPace {
    junction: f64,
    piece: f64,
}

impl Default for RejoinPace {
    fn default() -> Self {
        Self {
            junction: DEFAULT_JUNCTION,
            piece: DEFAULT_PIECE,
        }
    }
}

impl RejoinPace {
    pub fn new(junction: f64, piece: f64) -> Result<Self, Error> {
        for (value, setting) in [
            (junction, Setting::RejoinJunctionPace),
            (piece, Setting::RejoinPiecePace),
        ] {
            if !(value.is_finite() && value >= 0.0) {
                return Err(Error::Setting { setting, value });
            }
        }
        Ok(Self { junction, piece })
    }

    pub fn junction(self) -> f64 {
        self.junction
    }

    pub fn piece(self) -> f64 {
        self.piece
    }

    pub(crate) fn keeps(self, (ended, ended_role): (&Track, Role), (started, started_role): (&Track, Role)) -> bool {
        let (end, start) = (ended.last(), started.first());
        let junction = end.distance(start) / start.frame.abs_diff(end.frame) as f64;
        let piece = match (ended_role, started_role) {
            (Role::Kept, Role::Piece) => Some(started),
            (Role::Piece, Role::Kept) => Some(ended),
            _ => None,
        }
        .filter(|piece| piece.points().len() > 1);
        [(ended_role, tail(ended)), (started_role, head(started))]
            .into_iter()
            .filter(|(role, _)| *role == Role::Kept)
            .map(|(_, points)| speed(points))
            .all(|kept| {
                junction >= self.junction * kept && piece.is_none_or(|piece| speed(piece.points()) >= self.piece * kept)
            })
    }
}

fn speed(points: &[Detection]) -> f64 {
    let (x, y) = velocity(points);
    x.hypot(y)
}
