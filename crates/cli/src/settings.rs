use engine::settings::DetectSettings;
use media::decode::ActiveThreads;
use media::encode::Codec;
use serde::Serialize;
use tracking::settings::{StabilitySettings, TrackSettings};

use crate::options::Options;

#[cfg(test)]
mod tests;

#[derive(Debug, Serialize)]
pub(crate) struct Settings {
    #[serde(rename = "DetectConfig")]
    detect: DetectSettings,
    #[serde(rename = "TrackConfig")]
    track: TrackSettings,
    #[serde(rename = "StabilityConfig")]
    stability: StabilitySettings,
    #[serde(rename = "RenderConfig")]
    render: RenderSettings,
    threads: ThreadSettings,
}

#[derive(Debug, Serialize)]
struct RenderSettings {
    box_pad: u32,
    trail_s: f64,
    clip_margin_s: f64,
    max_tracks: usize,
    zoom: &'static str,
    zoom_below_area: f64,
    zoom_below_amplitude: f64,
    annotated: bool,
    encoders: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
struct ThreadSettings {
    decoding: usize,
    decoding_kind: DecodingKind,
    detection: usize,
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "lowercase")]
enum DecodingKind {
    Single,
    Frame,
    Slice,
}

impl Settings {
    pub(crate) fn of(options: &Options, encoders: &[Codec], decoding: ActiveThreads, detection: usize) -> Self {
        let (decoding_kind, decoding) = match decoding {
            ActiveThreads::Single => (DecodingKind::Single, 1),
            ActiveThreads::Frame(count) => (DecodingKind::Frame, count.get()),
            ActiveThreads::Slice(count) => (DecodingKind::Slice, count.get()),
        };
        Self {
            detect: DetectSettings::from(&options.detect),
            track: TrackSettings::from(options.track),
            stability: StabilitySettings::from(options.stability),
            render: RenderSettings {
                box_pad: options.render.box_pad(),
                trail_s: options.render.trail_s(),
                clip_margin_s: options.render.clip_margin_s(),
                max_tracks: options.render.max_tracks(),
                zoom: options.render.zoom().choice().name(),
                zoom_below_area: options.render.zoom().below_area(),
                zoom_below_amplitude: options.render.zoom().below_amplitude(),
                annotated: options.annotated,
                encoders: encoders.iter().map(|codec| encoder_name(*codec)).collect(),
            },
            threads: ThreadSettings {
                decoding,
                decoding_kind,
                detection,
            },
        }
    }
}

fn encoder_name(codec: Codec) -> &'static str {
    match codec {
        Codec::OpenH264 => "openh264",
        Codec::VideoToolbox => "videotoolbox",
    }
}
