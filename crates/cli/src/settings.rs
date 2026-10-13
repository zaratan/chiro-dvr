use std::num::NonZeroU32;

use engine::config::DetectConfig;
use engine::settings::DetectSettings;
use media::decode::ActiveThreads;
use media::encode::Codec;
use render::config::RenderConfig;
use serde::Serialize;
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

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
struct TrackSettings {
    max_jump: f64,
    max_gap: usize,
    min_hits: usize,
    min_travel: f64,
    twin_distance: f64,
    max_median_turn: f64,
}

#[derive(Debug, Serialize)]
struct StabilitySettings {
    max_blobs: u32,
    pad_s: f64,
}

#[derive(Debug, Serialize)]
struct RenderSettings {
    box_pad: u32,
    trail_s: f64,
    clip_margin_s: f64,
    max_tracks: usize,
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
    pub(crate) fn of(
        detect: &DetectConfig,
        track: TrackConfig,
        stability: StabilityConfig,
        render: RenderConfig,
        encoders: &[Codec],
        decoding: ActiveThreads,
        detection: usize,
    ) -> Self {
        let (decoding_kind, decoding) = match decoding {
            ActiveThreads::Single => (DecodingKind::Single, 1),
            ActiveThreads::Frame(count) => (DecodingKind::Frame, count.get()),
            ActiveThreads::Slice(count) => (DecodingKind::Slice, count.get()),
        };
        Self {
            detect: DetectSettings::from(detect),
            track: TrackSettings {
                max_jump: track.max_jump(),
                max_gap: track.max_gap().get(),
                min_hits: track.min_hits().get(),
                min_travel: track.min_travel(),
                twin_distance: track.twin_distance().unwrap_or(0.0),
                max_median_turn: track.max_median_turn(),
            },
            stability: StabilitySettings {
                max_blobs: stability.max_blobs().map_or(0, NonZeroU32::get),
                pad_s: stability.pad_s(),
            },
            render: RenderSettings {
                box_pad: render.box_pad(),
                trail_s: render.trail_s(),
                clip_margin_s: render.clip_margin_s(),
                max_tracks: render.max_tracks(),
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
