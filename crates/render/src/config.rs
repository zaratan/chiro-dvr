use types::rate::FrameRate;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{name} must be a finite number of seconds, at least 0, not {value}")]
    Seconds { name: &'static str, value: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderConfig {
    box_pad: u32,
    trail_s: f64,
    clip_margin_s: f64,
    max_tracks: usize,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            box_pad: 12,
            trail_s: 1.0,
            clip_margin_s: 1.5,
            max_tracks: 300,
        }
    }
}

impl RenderConfig {
    pub fn new(box_pad: u32, trail_s: f64, clip_margin_s: f64, max_tracks: usize) -> Result<Self, Error> {
        for (name, value) in [("trail_s", trail_s), ("clip_margin_s", clip_margin_s)] {
            if !(value.is_finite() && value >= 0.0) {
                return Err(Error::Seconds { name, value });
            }
        }
        Ok(Self {
            box_pad,
            trail_s,
            clip_margin_s,
            max_tracks,
        })
    }

    pub fn box_pad(self) -> u32 {
        self.box_pad
    }

    pub fn trail_s(self) -> f64 {
        self.trail_s
    }

    pub fn clip_margin_s(self) -> f64 {
        self.clip_margin_s
    }

    pub fn max_tracks(self) -> usize {
        self.max_tracks
    }

    pub fn renders_clips_for(self, track_count: usize) -> bool {
        self.max_tracks == 0 || track_count <= self.max_tracks
    }

    pub(crate) fn margin_frames(self, rate: FrameRate) -> usize {
        frames(self.clip_margin_s, rate)
    }

    pub(crate) fn trail_frames(self, rate: FrameRate) -> usize {
        frames(self.trail_s, rate)
    }
}

fn frames(seconds: f64, rate: FrameRate) -> usize {
    (seconds * rate.get().to_f64()).round() as usize
}
