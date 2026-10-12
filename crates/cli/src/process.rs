use std::fs;
use std::io;
use std::path::PathBuf;

use engine::config::DetectConfig;
use engine::detection::{self, Detected, detect};
use engine::input::Video;
use files::names::{Names, params_json};
use files::params::{Params, spans_in_seconds, write_params};
use files::table::write_tracks_csv;
use files::time::frame_seconds;
use media::decode::Threads;
use media::encode::Codec;
use media::encode::fallback::{self, Fallback};
use render::clips::{self, Clip, Rendered};
use render::config::RenderConfig;
use render::source::{self, Source};
use summary::header::Labels;
use summary::layout::Summary;
use tracking::exclusion::exclude;
use tracking::stability::StabilityConfig;
use tracking::track::{TrackConfig, track_detections};
use types::language::Language;
use types::timeline::{self, Timeline};
use types::track::Track;

use crate::jobs::Job;
use crate::overview::{self, Overview, write_overview};
use crate::settings::Settings;

#[cfg(test)]
mod tests;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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
    #[error("the results cannot be written")]
    Files(#[source] files::error::Error),
    #[error("the summary image cannot be written")]
    Overview(#[source] overview::Error),
    #[error("no encoder can write the clips")]
    Encoders(#[source] fallback::Error),
    #[error("cannot empty {}", path.display())]
    Clear {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("the clips cannot read the video")]
    Source(#[source] source::Error),
    #[error("the clips cannot be written")]
    Clips(#[source] clips::Error),
    #[error("{count} tracks, above {limit}: clips skipped")]
    TooManyTracks { count: usize, limit: usize },
}

#[derive(Debug, Clone, Copy)]
pub struct Versions<'a> {
    pub program: &'a str,
    pub ffmpeg: &'a str,
}

#[derive(Debug)]
pub struct Processed {
    pub tracks: Vec<Track>,
    pub csv: PathBuf,
    pub summaries: Vec<PathBuf>,
    pub rendered: Rendered,
}

pub fn process(
    job: &Job,
    language: Language,
    labels: &dyn Labels,
    threads: Threads,
    versions: Versions<'_>,
) -> Result<Processed, Error> {
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
    );
    let tracks = track_detections(&exclusion.detections, track).map_err(Error::Tracking)?;
    fs::create_dir_all(&job.dest).map_err(|source| Error::Folder {
        dest: job.dest.clone(),
        source,
    })?;
    let names = Names::new(language);
    let csv = names.tracks_csv(&job.dest, job.video.file_stem().unwrap_or_default());
    write_tracks_csv(&csv, &tracks, &timeline).map_err(Error::Files)?;
    let render = RenderConfig::default();
    let encoders = encoders(render, tracks.len(), reading.video())?;
    let settings = Settings::of(
        &config,
        track,
        stability,
        render,
        &encoders,
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
        labels,
        regions: config.osd_regions(),
    };
    let name = job.video.file_stem().unwrap_or_default().to_string_lossy();
    let summary = Summary {
        name: &name,
        tracks: &tracks,
        timeline: &timeline,
        ignored: &params.ignored_s,
        damaged: &params.damaged_s,
    };
    let summaries = write_overview(overview, &detected, &summary).map_err(Error::Overview)?;
    let clips = Clips {
        job,
        names,
        detected: &detected,
        timeline: &timeline,
        render,
    };
    let rendered = render_clips(&clips, &tracks, &encoders)?;
    Ok(Processed {
        tracks,
        csv,
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
    render: RenderConfig,
}

fn render_clips(clips: &Clips<'_>, tracks: &[Track], encoders: &[Codec]) -> Result<Rendered, Error> {
    let folder = clips.names.clips_dir(&clips.job.dest);
    match fs::remove_dir_all(&folder) {
        Err(source) if source.kind() != io::ErrorKind::NotFound => {
            return Err(Error::Clear { path: folder, source });
        }
        _ => {}
    }
    fs::create_dir_all(&folder).map_err(|source| Error::Folder {
        dest: folder.clone(),
        source,
    })?;
    if !clips.render.renders_clips_for(tracks.len()) {
        return Err(Error::TooManyTracks {
            count: tracks.len(),
            limit: clips.render.max_tracks(),
        });
    }
    let reading = clips.detected.reading();
    let errors = clips.detected.damage().error_frames();
    let source = Source::new(&clips.job.video, clips.timeline, &reading.key_frames(), errors).map_err(Error::Source)?;
    let wanted: Vec<Clip> = tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let start = frame_seconds(clips.timeline, track.first().frame);
            Clip {
                track: index,
                path: folder.join(clips.names.clip(track, start, tracks.len())),
            }
        })
        .collect();
    clips::render(&source, tracks, &wanted, clips.render, encoders).map_err(Error::Clips)
}
