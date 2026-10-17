use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use draw::color::Rgb;
use engine::detection::{self, Detected, detect};
use engine::input::Video;
use files::aside::Stamp;
use files::names::{Names, params_json};
use files::params::{Finish, Params, language_code, local_date, spans_in_seconds, write_params};
use files::previous::{Kept, remove_previous_outputs};
use files::table::{Headings, write_tracks_csv};
use files::time::frame_seconds;
use jiff::Zoned;
use media::decode::Threads;
use media::encode::Codec;
use media::encode::fallback::{self, Fallback};
use render::clips::{self, Clip, Outputs, Rendered};
use render::config::RenderConfig;
use render::source::{self, Source};
use render::windows::{Window, clip_windows};
use summary::header::Labels;
use summary::layout::Summary;
use tracking::exclusion::{Exclusion, exclude};
use tracking::track::track_detections;
use types::frame::FrameIndex;
use types::language::Language;
use types::timeline::{self, Timeline};
use types::track::Track;

use crate::disk;
use crate::jobs::Job;
use crate::listing::listed;
use crate::options::Options;
use crate::overview::{self, Overview, write_overview};
use crate::progress::{Progress, Step};
use crate::settings::Settings;
use crate::space::{Rendering, render_estimate};

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
    #[error("cannot read the size of {}", path.display())]
    Video {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot create {}", dest.display())]
    Folder {
        dest: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("the results of an earlier run cannot be removed or set aside")]
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
    #[error("{needed} bytes are needed on the disk of {}, {free} are free", dest.display())]
    DiskSpace { dest: PathBuf, needed: u64, free: u64 },
    #[error("{count} tracks, above {limit}: clips skipped")]
    TooManyTracks {
        count: usize,
        limit: usize,
        annotated: bool,
        extent: Extent,
        kept: Vec<Kept>,
    },
}

pub trait Wording: Labels + Headings {
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
    pub extent: Extent,
    pub csv: Option<PathBuf>,
    pub kept: Vec<Kept>,
    pub summaries: Vec<PathBuf>,
    pub rendered: Rendered,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Extent {
    pub length_s: f64,
    pub not_analysed_s: f64,
}

pub fn extent(timeline: &Timeline, exclusion: &Exclusion) -> Extent {
    let end = FrameIndex::new(timeline.frame_count());
    Extent {
        length_s: frame_seconds(timeline, end) - frame_seconds(timeline, FrameIndex::new(0)),
        not_analysed_s: timeline.rate().seconds(FrameIndex::new(exclusion.ignored_frame_count)),
    }
}

pub fn process(
    job: &Job,
    wording: &dyn Wording,
    progress: &dyn Progress,
    threads: Threads,
    versions: Versions<'_>,
    options: &Options,
) -> Result<Processed, Error> {
    let started = Instant::now();
    progress.step(Step::Detection).map_err(Error::Progress)?;
    let detected = detect(&job.video, threads, &options.detect).map_err(Error::Detection)?;
    processed_since(started, job, &detected, wording, progress, versions, options)
}

#[derive(Debug)]
pub struct Tracked {
    pub timeline: Timeline,
    pub exclusion: Exclusion,
    pub tracks: Vec<Track>,
}

pub fn tracked(detected: &Detected, options: &Options) -> Result<Tracked, Error> {
    let reading = detected.reading();
    let rate = reading.video().frame_rate();
    let pts = reading.frames().iter().map(|frame| frame.pts()).collect();
    let timeline = Timeline::new(pts, reading.video().time_base(), rate).map_err(Error::Timeline)?;
    let damage = detected.damage();
    let exclusion = exclude(
        detected.detections(),
        damage.spans().to_vec(),
        damage.reported_frames(),
        rate,
        options.stability,
    );
    let tracks = track_detections(&exclusion.detections, options.track).map_err(Error::Tracking)?;
    Ok(Tracked {
        timeline,
        exclusion,
        tracks,
    })
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
    processed_since(Instant::now(), job, detected, wording, progress, versions, options)
}

fn processed_since(
    started: Instant,
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
    let reported = detected.damage().reported_frames();
    progress.step(Step::Tracking).map_err(Error::Progress)?;
    let Tracked {
        timeline,
        exclusion,
        tracks,
    } = tracked(detected, options)?;
    let video_bytes = video_bytes(&job.video)?;
    fs::create_dir_all(&job.dest).map_err(|source| Error::Folder {
        dest: job.dest.clone(),
        source,
    })?;
    let names = Names::new(wording.language());
    let stem = job.video.file_stem().unwrap_or_default();
    let mut kept = remove_previous_outputs(&job.dest, stem, names, wording, Stamp::now()).map_err(Error::Previous)?;
    let csv = write_table(
        names.tracks_csv(&job.dest, stem),
        &tracks,
        &timeline,
        wording,
        &mut kept,
    )?;
    let trial = encoders(render, tracks.len(), reading.video());
    let settings = Settings::of(
        options,
        trial.as_deref().unwrap_or_default(),
        detected.active_threads(),
        detected.detection_threads(),
    );
    let ignored_s = spans_in_seconds(&exclusion.unstable, &timeline);
    let damaged_s = spans_in_seconds(&exclusion.damaged, &timeline);
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
        ignored: &ignored_s,
        damaged: &damaged_s,
    };
    progress.step(Step::Summary).map_err(Error::Progress)?;
    let drawn = write_overview(overview, detected, &summary).map_err(Error::Overview)?;
    let colors = drawn.colors(&tracks);
    let clips = Clips {
        job,
        names,
        detected,
        colors: &colors,
        timeline: &timeline,
        options,
        extent: extent(&timeline, &exclusion),
        kept: &kept,
    };
    room_for_clips(&clips, &tracks, video_bytes)?;
    progress.step(Step::Clips).map_err(Error::Progress)?;
    let written = render_clips(&clips, &tracks, trial);
    if let Some(state) = finish(&written) {
        let outputs = written.as_ref().ok().map(|written| &written.outputs);
        let params = Params {
            video: &job.video,
            fps: rate.get().to_f64(),
            settings: &settings,
            ignored_s,
            damaged_s,
            damaged_frames: reported,
            version: versions.program,
            ffmpeg: versions.ffmpeg,
            state,
            track_count: tracks.len(),
            length_s: clips.extent.length_s,
            not_analysed_s: clips.extent.not_analysed_s,
            elapsed_s: started.elapsed().as_secs_f64(),
            video_bytes,
            language: language_code(wording.language()),
            date: local_date(&Zoned::now()),
            tracks: listed(&job.dest, &tracks, &timeline, &drawn.painted, outputs),
        };
        write_params(&params_json(&job.dest), &params).map_err(Error::Files)?;
    }
    let Written { rendered, .. } = written?;
    Ok(Processed {
        tracks,
        extent: clips.extent,
        csv,
        kept,
        summaries: drawn.pages,
        rendered,
    })
}

#[derive(Debug)]
struct Written {
    rendered: Rendered,
    outputs: Outputs,
}

fn finish(written: &Result<Written, Error>) -> Option<Finish> {
    match written {
        Ok(_) => Some(Finish::Done),
        Err(Error::TooManyTracks { .. }) => Some(Finish::TooManyTracks),
        Err(_) => None,
    }
}

fn video_bytes(video: &Path) -> Result<u64, Error> {
    let metadata = fs::metadata(video).map_err(|source| Error::Video {
        path: video.to_path_buf(),
        source,
    })?;
    Ok(metadata.len())
}

fn write_table(
    csv: PathBuf,
    tracks: &[Track],
    timeline: &Timeline,
    wording: &dyn Wording,
    kept: &mut Vec<Kept>,
) -> Result<Option<PathBuf>, Error> {
    if kept.iter().any(Kept::blocks_our_table) {
        return Ok(None);
    }
    match write_tracks_csv(&csv, tracks, timeline, wording.language(), wording) {
        Ok(()) => Ok(Some(csv)),
        Err(files::error::Error::Taken { path }) => {
            kept.push(Kept::Left { path, ours: true });
            Ok(None)
        }
        Err(error) => Err(Error::Files(error)),
    }
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
    colors: &'a [Option<Rgb>],
    timeline: &'a Timeline,
    options: &'a Options,
    extent: Extent,
    kept: &'a [Kept],
}

fn room_for_clips(clips: &Clips<'_>, tracks: &[Track], video_bytes: u64) -> Result<(), Error> {
    let render = clips.options.render;
    let rate = clips.timeline.rate();
    if !render.renders_clips_for(tracks.len()) {
        return Ok(());
    }
    let seconds = |window: &Window| rate.seconds(FrameIndex::new(window.span.frame_count()));
    let windows = clip_windows(tracks, clips.timeline.frame_count(), render, rate);
    let zoomed = |window: &&Window| tracks.get(window.track).is_some_and(|track| render.zoom().wants(track));
    let rendering = Rendering {
        clip_s: windows.iter().map(seconds).sum(),
        zoom_s: windows.iter().filter(zoomed).map(seconds).sum(),
        annotated: clips.options.annotated,
    };
    let needed = render_estimate(video_bytes, clips.extent.length_s, rendering);
    match disk::volume(&clips.job.dest) {
        Some(volume) if needed > volume.free => Err(Error::DiskSpace {
            dest: clips.job.dest.clone(),
            needed,
            free: volume.free,
        }),
        _ => Ok(()),
    }
}

fn render_clips(clips: &Clips<'_>, tracks: &[Track], trial: Result<Vec<Codec>, Error>) -> Result<Written, Error> {
    let stem = clips.job.video.file_stem().unwrap_or_default();
    let annotated = clips.names.annotated_video(&clips.job.dest, stem);
    let render = clips.options.render;
    let folder = clips.names.clips_dir(&clips.job.dest);
    if !render.renders_clips_for(tracks.len()) {
        return Err(Error::TooManyTracks {
            count: tracks.len(),
            limit: render.max_tracks(),
            annotated: clips.options.annotated,
            extent: clips.extent,
            kept: clips.kept.to_vec(),
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
    if !outputs.clips.is_empty() {
        fs::create_dir_all(&folder).map_err(|source| Error::Folder {
            dest: folder.clone(),
            source,
        })?;
    }
    let rendered = clips::render(&source, tracks, clips.colors, &outputs, render, &encoders).map_err(Error::Clips)?;
    Ok(Written { rendered, outputs })
}
