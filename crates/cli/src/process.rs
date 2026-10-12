use std::fs;
use std::io;
use std::path::PathBuf;

use engine::config::DetectConfig;
use engine::detection::{self, detect};
use files::names::{Names, params_json};
use files::params::{Params, spans_in_seconds, write_params};
use files::table::write_tracks_csv;
use media::decode::Threads;
use tracking::exclusion::exclude;
use tracking::stability::StabilityConfig;
use tracking::track::{TrackConfig, track_detections};
use types::language::Language;
use types::timeline::{self, Timeline};
use types::track::Track;

use crate::jobs::Job;
use crate::settings::Settings;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the detection failed")]
    Detection(#[source] detection::Error),
    #[error("the timeline cannot be built")]
    Timeline(#[source] timeline::Error),
    #[error("the exclusion of damaged and unstable frames failed")]
    Exclusion(#[source] tracking::error::Error),
    #[error("the tracking failed")]
    Tracking(#[source] tracking::error::Error),
    #[error("cannot create {}", dest.display())]
    Folder {
        dest: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("the results cannot be written")]
    Files(#[source] files::error::Error),
}

#[derive(Debug, Clone, Copy)]
pub struct Versions<'a> {
    pub program: &'a str,
    pub ffmpeg: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Processed {
    pub tracks: Vec<Track>,
    pub csv: PathBuf,
}

pub fn process(job: &Job, language: Language, threads: Threads, versions: Versions<'_>) -> Result<Processed, Error> {
    let config = DetectConfig::default();
    let detected = detect(&job.video, threads, &config).map_err(Error::Detection)?;
    let reading = detected.reading();
    let rate = reading.video().frame_rate();
    let pts = reading.frames().iter().map(|frame| frame.pts()).collect();
    let timeline = Timeline::new(pts, reading.video().time_base(), rate).map_err(Error::Timeline)?;
    let damage = detected.damage();
    let reported = damage.reported_frames();
    let track = TrackConfig::default();
    let stability = StabilityConfig::default();
    let exclusion = exclude(
        detected.detections(),
        damage.spans().to_vec(),
        reported,
        rate,
        stability,
    )
    .map_err(Error::Exclusion)?;
    let tracks = track_detections(&exclusion.detections, track).map_err(Error::Tracking)?;
    fs::create_dir_all(&job.dest).map_err(|source| Error::Folder {
        dest: job.dest.clone(),
        source,
    })?;
    let csv = Names::new(language).tracks_csv(&job.dest, job.video.file_stem().unwrap_or_default());
    write_tracks_csv(&csv, &tracks, &timeline).map_err(Error::Files)?;
    let settings = Settings::of(
        &config,
        track,
        stability,
        detected.active_threads(),
        detected.detection_threads(),
    );
    let params = Params {
        video: &job.video,
        fps: rate.get().to_f64(),
        settings: &settings,
        ignored_s: spans_in_seconds(&exclusion.unstable, &timeline),
        damaged_s: spans_in_seconds(&exclusion.damaged, &timeline),
        damaged_frames: reported,
        version: versions.program,
        ffmpeg: versions.ffmpeg,
    };
    write_params(&params_json(&job.dest), &params).map_err(Error::Files)?;
    Ok(Processed { tracks, csv })
}
