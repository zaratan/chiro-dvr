use types::rate::FrameRate;
use types::track::Track;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{name} must be a finite number of seconds, at least 0, not {value}")]
    Seconds { name: &'static str, value: f64 },
    #[error("{name} must be a finite number above 0, not {value}")]
    Threshold { name: &'static str, value: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Zoom {
    Auto,
    All,
    Off,
}

impl Zoom {
    pub const CHOICES: [Self; 3] = [Self::Auto, Self::All, Self::Off];

    pub fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::All => "all",
            Self::Off => "none",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZoomConfig {
    choice: Zoom,
    below_area: f64,
    below_amplitude: f64,
}

impl Default for ZoomConfig {
    fn default() -> Self {
        Self {
            choice: Zoom::Auto,
            below_area: 100.0,
            below_amplitude: 40.0,
        }
    }
}

impl ZoomConfig {
    pub fn new(choice: Zoom, below_area: f64, below_amplitude: f64) -> Result<Self, Error> {
        for (name, value) in [
            ("zoom_below_area", below_area),
            ("zoom_below_amplitude", below_amplitude),
        ] {
            if !(value.is_finite() && value > 0.0) {
                return Err(Error::Threshold { name, value });
            }
        }
        Ok(Self {
            choice,
            below_area,
            below_amplitude,
        })
    }

    pub fn choosing(choice: Zoom) -> Self {
        Self {
            choice,
            ..Self::default()
        }
    }

    pub fn choice(self) -> Zoom {
        self.choice
    }

    pub fn below_area(self) -> f64 {
        self.below_area
    }

    pub fn below_amplitude(self) -> f64 {
        self.below_amplitude
    }

    pub fn wants(self, track: &Track) -> bool {
        match self.choice {
            Zoom::All => true,
            Zoom::Off => false,
            Zoom::Auto => track.max_area() < self.below_area || track.peak_amplitude() < self.below_amplitude,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderConfig {
    box_pad: u32,
    trail_s: f64,
    clip_margin_s: f64,
    max_tracks: usize,
    zoom: ZoomConfig,
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            box_pad: 12,
            trail_s: 1.0,
            clip_margin_s: 1.5,
            max_tracks: 300,
            zoom: ZoomConfig::default(),
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
            zoom: ZoomConfig::default(),
        })
    }

    #[must_use]
    pub fn with_zoom(self, zoom: ZoomConfig) -> Self {
        Self { zoom, ..self }
    }

    pub fn zoom(self) -> ZoomConfig {
        self.zoom
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
