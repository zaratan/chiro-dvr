use serde::{Deserialize, Serialize};
use types::detection::Detection;
use types::frame::FrameIndex;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub(crate) struct Measures {
    x: f64,
    y: f64,
    left: f64,
    top: f64,
    width: f64,
    height: f64,
    area: f64,
    amplitude: f64,
}

impl Measures {
    pub(crate) fn of(detection: &Detection) -> Self {
        Self {
            x: detection.x,
            y: detection.y,
            left: detection.left,
            top: detection.top,
            width: detection.width,
            height: detection.height,
            area: detection.area,
            amplitude: detection.amplitude,
        }
    }

    pub(crate) fn at(self, frame: FrameIndex) -> Detection {
        Detection {
            frame,
            x: self.x,
            y: self.y,
            left: self.left,
            top: self.top,
            width: self.width,
            height: self.height,
            area: self.area,
            amplitude: self.amplitude,
        }
    }
}
