use std::env::consts::{ARCH, OS};

use engine::config::DetectConfig;
use engine::detection::Detected;
use engine::input::Video;
use engine::settings::DetectSettings;
use media::decode::ActiveThreads;
use reference::video::{Counts, Decoder, DecoderThreads, Losses, VideoSummary};
use serde::Serialize;
use serde_json::{Map, Value};
use tracking::exclusion::Exclusion;
use tracking::settings::{StabilitySettings, TrackSettings};
use tracking::stability::StabilityConfig;
use tracking::track::{TrackConfig, track_detections};
use types::rational::Rational;
use types::track::Track;

use crate::error::Error;
use crate::provenance::Build;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq)]
pub struct Followed {
    pub exclusion: Exclusion,
    pub tracks: Vec<Track>,
}

pub fn followed(detected: &Detected) -> Result<Followed, tracking::error::Error> {
    let damage = detected.damage();
    let exclusion = tracking::exclusion::exclude(
        detected.detections(),
        damage.spans().to_vec(),
        damage.reported_frames(),
        detected.reading().video().frame_rate(),
        StabilityConfig::default(),
    );
    let tracks = track_detections(&exclusion.detections, TrackConfig::default())?;
    Ok(Followed { exclusion, tracks })
}

pub fn same_stream(probed: &VideoSummary, video: &Video) -> Result<(), Error> {
    let stream = &probed.stream;
    let size = video.size();
    let expected = described(stream.width, stream.height, stream.avg_frame_rate, stream.time_base);
    let found = described(
        size.width().get(),
        size.height().get(),
        video.frame_rate().get(),
        video.time_base(),
    );
    if found == expected {
        Ok(())
    } else {
        Err(Error::Stream {
            video: probed.video.name.clone(),
            expected,
            found,
        })
    }
}

fn described(width: u32, height: u32, rate: Rational, time_base: Rational) -> String {
    format!("{width}×{height} at {rate} frames per second, time base {time_base}")
}

pub fn summary(
    probed: VideoSummary,
    detected: &Detected,
    followed: &Followed,
    config: &DetectConfig,
    build: &Build,
) -> Result<VideoSummary, Error> {
    let video = probed.video.name.clone();
    let reading = detected.reading();
    let damage = detected.damage();
    let losses = reading.losses();
    let decoder = Decoder {
        error_frames: damage.error_frames().to_vec(),
        gap_frames: damage.jump_frames().to_vec(),
        losses: Losses {
            before_first: losses.before_first,
            after_last: losses.after_last,
        },
        threads: threads(detected.active_threads()),
    };
    let recorded = sections(config, build).map_err(|source| Error::Section { video, source })?;
    Ok(VideoSummary {
        background_half_window: detected.half_window().get(),
        damaged_spans: damage.spans().to_vec(),
        unstable_spans: followed.exclusion.unstable.clone(),
        counts: Counts {
            frames: reading.frames().len(),
            detections: detected.detections().detection_count(),
            detections_kept: followed.exclusion.detections.detection_count(),
            damaged_frames_reported: damage.reported_frames(),
            tracks: followed.tracks.len(),
            track_points: followed.tracks.iter().map(|track| track.points().len()).sum(),
        },
        recorded,
        ..probed.decoded(build.release(), decoder)
    })
}

fn threads(active: ActiveThreads) -> DecoderThreads {
    match active {
        ActiveThreads::Single => DecoderThreads::Single,
        ActiveThreads::Frame(count) => DecoderThreads::Frame(count),
        ActiveThreads::Slice(count) => DecoderThreads::Slice(count),
    }
}

fn sections(config: &DetectConfig, build: &Build) -> Result<Map<String, Value>, serde_json::Error> {
    let settings = object([
        ("DetectConfig", value(&DetectSettings::from(config))?),
        ("TrackConfig", value(&TrackSettings::from(TrackConfig::default()))?),
        (
            "StabilityConfig",
            value(&StabilitySettings::from(StabilityConfig::default()))?,
        ),
    ]);
    let versions = object([
        ("batdetect", Value::from(build.version())),
        ("commit", Value::from(build.commit())),
        ("rustc", Value::from(build.rustc())),
        ("ffmpeg", Value::from(media::library::version().into_owned())),
        ("system", Value::from(format!("{OS}-{ARCH}"))),
    ]);
    Ok(Map::from_iter([
        ("config".to_owned(), Value::Object(settings)),
        ("versions".to_owned(), Value::Object(versions)),
    ]))
}

fn value(section: &impl Serialize) -> Result<Value, serde_json::Error> {
    serde_json::to_value(section)
}

fn object<const N: usize>(fields: [(&str, Value); N]) -> Map<String, Value> {
    fields.into_iter().map(|(key, value)| (key.to_owned(), value)).collect()
}
