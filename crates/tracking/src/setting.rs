use std::fmt;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Setting {
    PadS,
    MaxJump,
    MinTravel,
    TwinDistance,
    MaxMedianTurn,
}

impl fmt::Display for Setting {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PadS => "pad_s",
            Self::MaxJump => "max_jump",
            Self::MinTravel => "min_travel",
            Self::TwinDistance => "twin_distance",
            Self::MaxMedianTurn => "max_median_turn",
        })
    }
}
