use std::fs;
use std::io;
use std::path::PathBuf;

use engine::detection::{self, Detected, detect};
use engine::input::Video;
use files::names::{Names, params_json};
use files::params::{Params, spans_in_seconds, write_params};
use files::previous::{Kept, remove_previous_outputs};
use files::table::write_tracks_csv;
use files::time::frame_seconds;
use media::decode::Threads;
use media::encode::Codec;
use media::encode::fallback::{self, Fallback};
use render::clips::{self, Clip, Outputs, Rendered};
use render::config::RenderConfig;
use render::source::{self, Source};
use summary::header::Labels;
use summary::layout::Summary;
use tracking::exclusion::exclude;
use tracking::track::track_detections;
use types::language::Language;
use types::timeline::{self, Timeline};
use types::track::Track;

use crate::jobs::Job;
use crate::options::Options;
use crate::overview::{self, Overview, write_overview};
use crate::progress::{Progress, Step};
use crate::settings::Settings;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the progress cannot be shown")]
    Progress(#[source] io::Error),
    #[error("the detection failed")]
    Detection(#[source] detection::Error),
    #[error("the timeline cannot be built")]
    Timeline(#[source] timeline::Error),
    #[error("the tracking failed")]
    Tracking(#[source] tracking::error::Error),
    #[error("cannot create {}", dest.display())]
    Folder {
        dest: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("the results of an earlier run cannot be removed")]
    Previous(#[source] files::error::Error),
    #[error("the results cannot be written")]
    Files(#[source] files::error::Error),
    #[error("the summary image cannot be written")]
    Overview(#[source] overview::Error),
    #[error("no encoder can write the clips")]
    Encoders(#[source] fallback::Error),
    #[error("the clips cannot read the video")]
    Source(#[source] source::Error),
    #[error("the clips cannot be written")]
    Clips(#[source] clips::Error),
    #[error("{count} tracks, above {limit}: clips skipped")]
    TooManyTracks {
        count: usize,
        limit: usize,
        annotated: bool,
    },
}

pub trait Wording: Labels {
    fn language(&self) -> Language;
}

#[derive(Debug, Clone, Copy)]
pub struct Versions<'a> {
    pub program: &'a str,
    pub ffmpeg: &'a str,
}

#[derive(Debug)]
pub struct Processed {
    pub tracks: Vec<Track>,
    pub csv: Option<PathBuf>,
    pub kept: Vec<Kept>,
    pub summaries: Vec<PathBuf>,
    pub rendered: Rendered,
}

pub fn process(
    job: &Job,
    wording: &dyn Wording,
    progress: &dyn Progress,
    threads: Threads,
    versions: Versions<'_>,
    options: &Options,
) -> Result<Processed, Error> {
    progress.step(Step::Detection).map_err(Error::Progress)?;
    let detected = detect(&job.video, threads, &options.detect).map_err(Error::Detection)?;
    process_detected(job, &detected, wording, progress, versions, options)
}

/// `detected` must come from `options.detect`: the parameters and the overview describe those settings.
pub fn process_detected(
    job: &Job,
    detected: &Detected,
    wording: &dyn Wording,
    progress: &dyn Progress,
    versions: Versions<'_>,
    options: &Options,
) -> Result<Processed, Error> {
    let render = options.render;
    let config = &options.detect;
    let reading = detected.reading();
    let rate = reading.video().frame_rate();
    let pts = reading.frames().iter().map(|frame| frame.pts()).collect();
    let timeline = Timeline::new(pts, reading.video().time_base(), rate).map_err(Error::Timeline)?;
    let damage = detected.damage();
    let reported = damage.reported_frames();
    let (track, stability) = (options.track, options.stability);
    progress.step(Step::Tracking).map_err(Error::Progress)?;
    let exclusion = exclude(
        detected.detections(),
        damage.spans().to_vec(),
        reported,
        rate,
        stability,
    );
    let tracks = track_detections(&exclusion.detections, track).map_err(Error::Tracking)?;
    fs::create_dir_all(&job.dest).map_err(|source| Error::Folder {
        dest: job.dest.clone(),
        source,
    })?;
    let names = Names::new(wording.language());
    let stem = job.video.file_stem().unwrap_or_default();
    let mut kept = remove_previous_outputs(&job.dest, stem, names).map_err(Error::Previous)?;
    let csv = names.tracks_csv(&job.dest, stem);
    let csv = if kept.iter().any(|table| table.ours) {
        None
    } else {
        match write_tracks_csv(&csv, &tracks, &timeline) {
            Ok(()) => Some(csv),
            Err(files::error::Error::Taken { path }) => {
                kept.push(Kept { path, ours: true });
                None
            }
            Err(error) => return Err(Error::Files(error)),
        }
    };
    let trial = encoders(render, tracks.len(), reading.video());
    let settings = Settings::of(
        options,
        trial.as_deref().unwrap_or_default(),
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
    let overview = Overview {
        video: &job.video,
        dest: &job.dest,
        names,
        labels: wording,
        regions: config.osd_regions(),
    };
    let name = stem.to_string_lossy();
    let summary = Summary {
        name: &name,
        tracks: &tracks,
        timeline: &timeline,
        ignored: &params.ignored_s,
        damaged: &params.damaged_s,
    };
    progress.step(Step::Summary).map_err(Error::Progress)?;
    let summaries = write_overview(overview, detected, &summary).map_err(Error::Overview)?;
    let clips = Clips {
        job,
        names,
        detected,
        timeline: &timeline,
        options,
    };
    progress.step(Step::Clips).map_err(Error::Progress)?;
    let rendered = render_clips(&clips, &tracks, trial)?;
    Ok(Processed {
        tracks,
        csv,
        kept,
        summaries,
        rendered,
    })
}

fn encoders(render: RenderConfig, track_count: usize, video: &Video) -> Result<Vec<Codec>, Error> {
    if !render.renders_clips_for(track_count) {
        return Ok(Vec::new());
    }
    let size = video.size();
    let fallback = Fallback::for_this_platform()
        .trial(size.width().get(), size.height().get(), video.frame_rate().get())
        .map_err(Error::Encoders)?;
    Ok(fallback.codecs().collect())
}

#[derive(Debug)]
struct Clips<'a> {
    job: &'a Job,
    names: Names,
    detected: &'a Detected,
    timeline: &'a Timeline,
    options: &'a Options,
}

fn render_clips(clips: &Clips<'_>, tracks: &[Track], trial: Result<Vec<Codec>, Error>) -> Result<Rendered, Error> {
    let stem = clips.job.video.file_stem().unwrap_or_default();
    let annotated = clips.names.annotated_video(&clips.job.dest, stem);
    let render = clips.options.render;
    let folder = clips.names.clips_dir(&clips.job.dest);
    fs::create_dir_all(&folder).map_err(|source| Error::Folder {
        dest: folder.clone(),
        source,
    })?;
    if !render.renders_clips_for(tracks.len()) {
        return Err(Error::TooManyTracks {
            count: tracks.len(),
            limit: render.max_tracks(),
            annotated: clips.options.annotated,
        });
    }
    let encoders = trial?;
    let reading = clips.detected.reading();
    let errors = clips.detected.damage().error_frames();
    let source = Source::new(&clips.job.video, clips.timeline, &reading.key_frames(), errors).map_err(Error::Source)?;
    let start = |track: &Track| frame_seconds(clips.timeline, track.first().frame);
    let outputs = Outputs {
        clips: tracks
            .iter()
            .enumerate()
            .map(|(index, track)| Clip {
                track: index,
                path: folder.join(clips.names.clip(track, start(track), tracks.len())),
            })
            .collect(),
        zooms: tracks
            .iter()
            .enumerate()
            .filter(|(_, track)| render.zoom().wants(track))
            .map(|(index, track)| Clip {
                track: index,
                path: folder.join(clips.names.zoomed_clip(track, start(track), tracks.len())),
            })
            .collect(),
        annotated: clips.options.annotated.then_some(annotated),
    };
    clips::render(&source, tracks, &outputs, render, &encoders).map_err(Error::Clips)
}
