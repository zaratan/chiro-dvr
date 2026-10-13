use std::fs;
use std::io;
use std::num::{NonZeroU32, NonZeroUsize};
use std::path::{Path, PathBuf};

use engine::config::DetectConfig;
use engine::settings::DetectSettings;
use reference::bench::{INJECTED_FILE, TARGETS_FILE, read_detections, read_targets, write_detections, write_targets};
use serde::{Deserialize, Serialize};
use tracking::settings::{StabilitySettings, TrackSettings};
use types::frame::FrameIndex;
use types::rate::{self, FrameRate};
use types::rational::Rational;
use types::size::SourceSize;
use types::span::{self, Span};

use crate::recorded;
use crate::run::{Provenance, Run, Video};
use crate::synthetic::injection::Injected;
use crate::synthetic::sampling::Sampling;
use crate::synthetic::{self, Class, UnknownMotion};

#[cfg(test)]
mod tests;

pub(crate) const RUN_FILE: &str = "run.json";
pub(crate) const REFERENCE_FILE: &str = "reference.jsonl.gz";
pub(crate) const FORMAT_VERSION: u32 = 1;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot create the folder {}", path.display())]
    Folder {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot read {}", path.display())]
    Read {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot write {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("{} does not describe a bench run", path.display())]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("{} has format version {found}, only version {FORMAT_VERSION} is read", path.display())]
    Version { path: PathBuf, found: u32 },
    #[error("{} describes a frame of {width}×{height} pixels", path.display())]
    Size { path: PathBuf, width: u32, height: u32 },
    #[error("{} describes a frame rate that is not positive", path.display())]
    Rate {
        path: PathBuf,
        #[source]
        source: rate::Error,
    },
    #[error("{} describes a damaged span that ends before it starts", path.display())]
    Span {
        path: PathBuf,
        #[source]
        source: span::Error,
    },
    #[error("{} describes a background half window of 0 frames", path.display())]
    HalfWindow { path: PathBuf },
    #[error("{} describes a class of targets that is not valid", path.display())]
    Class {
        path: PathBuf,
        #[source]
        source: synthetic::Error,
    },
    #[error("{} describes a class of unknown motion", path.display())]
    Motion {
        path: PathBuf,
        #[source]
        source: UnknownMotion,
    },
    #[error("{} describes no target per class", path.display())]
    PerClass { path: PathBuf },
    #[error(transparent)]
    File(#[from] reference::error::Error),
    #[error("cannot encode {}", path.display())]
    Encode {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("{} describes detection settings that are not valid", path.display())]
    Detect {
        path: PathBuf,
        #[source]
        source: engine::settings::Error,
    },
    #[error("{} holds target {id}, which ends beyond the {frames} frames of the run", path.display())]
    Beyond { path: PathBuf, id: usize, frames: usize },
    #[error("{} holds a target that cannot be replayed", path.display())]
    Target {
        path: PathBuf,
        #[source]
        source: recorded::Error,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RunFile {
    format_version: u32,
    video: PathBuf,
    width: u32,
    height: u32,
    frame_rate: Rational,
    damaged_spans: Vec<[FrameIndex; 2]>,
    reported_frames: usize,
    half_window: usize,
    detect: DetectSettings,
    targets: TargetsFile,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase")]
enum TargetsFile {
    Drawn {
        seed: u64,
        classes: Vec<ClassFile>,
        per_class: usize,
        min_distance: f64,
        track: TrackSettings,
        stability: StabilitySettings,
    },
    Replayed(PathBuf),
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ClassFile {
    amplitude: f64,
    sigma: f64,
    motion: String,
}

pub fn write(run: &Run, folder: &Path) -> Result<(), Error> {
    fs::create_dir_all(folder).map_err(|source| Error::Folder {
        path: folder.to_path_buf(),
        source,
    })?;
    let path = folder.join(RUN_FILE);
    let mut text = serde_json::to_string_pretty(&run_file(run)).map_err(|source| Error::Encode {
        path: path.clone(),
        source,
    })?;
    text.push('\n');
    fs::write(&path, text).map_err(|source| Error::Write { path, source })?;
    write_detections(&folder.join(REFERENCE_FILE), &run.reference)?;
    write_detections(&folder.join(INJECTED_FILE), &run.injected)?;
    let lines: Vec<_> = run.targets.iter().map(recorded::line).collect();
    Ok(write_targets(&folder.join(TARGETS_FILE), &lines)?)
}

pub fn read(folder: &Path) -> Result<Run, Error> {
    let path = folder.join(RUN_FILE);
    let text = fs::read_to_string(&path).map_err(|source| Error::Read {
        path: path.clone(),
        source,
    })?;
    let file: RunFile = serde_json::from_str(&text).map_err(|source| Error::Json {
        path: path.clone(),
        source,
    })?;
    if file.format_version != FORMAT_VERSION {
        return Err(Error::Version {
            path,
            found: file.format_version,
        });
    }
    let targets_path = folder.join(TARGETS_FILE);
    let targets: Vec<Injected> =
        recorded::all_injected(&read_targets(&targets_path)?).map_err(|source| Error::Target {
            path: targets_path.clone(),
            source,
        })?;
    DetectConfig::try_from(&file.detect).map_err(|source| Error::Detect {
        path: path.clone(),
        source,
    })?;
    let injected = read_detections(&folder.join(INJECTED_FILE))?;
    if let Some(beyond) = targets
        .iter()
        .find(|placed| placed.target.end().get() >= injected.frame_count())
    {
        return Err(Error::Beyond {
            path: targets_path,
            id: beyond.target.id(),
            frames: injected.frame_count(),
        });
    }
    Ok(Run {
        video: video(&file, &path)?,
        provenance: provenance(file.targets, &path)?,
        detect: file.detect,
        reference: read_detections(&folder.join(REFERENCE_FILE))?,
        injected,
        targets,
    })
}

fn run_file(run: &Run) -> RunFile {
    let video = &run.video;
    RunFile {
        format_version: FORMAT_VERSION,
        video: video.path.clone(),
        width: video.size.width().get(),
        height: video.size.height().get(),
        frame_rate: video.rate.get(),
        damaged_spans: video.damaged.iter().map(|span| [span.first(), span.last()]).collect(),
        reported_frames: video.reported_frames,
        half_window: video.half_window.get(),
        detect: run.detect.clone(),
        targets: match &run.provenance {
            Provenance::Drawn {
                sampling,
                track,
                stability,
            } => TargetsFile::Drawn {
                seed: sampling.seed,
                classes: sampling
                    .classes
                    .iter()
                    .map(|class| ClassFile {
                        amplitude: class.amplitude(),
                        sigma: class.sigma(),
                        motion: class.motion().name().to_owned(),
                    })
                    .collect(),
                per_class: sampling.per_class.get(),
                min_distance: sampling.min_distance,
                track: *track,
                stability: *stability,
            },
            Provenance::Replayed(path) => TargetsFile::Replayed(path.clone()),
        },
    }
}

fn video(file: &RunFile, path: &Path) -> Result<Video, Error> {
    let size = NonZeroU32::new(file.width)
        .zip(NonZeroU32::new(file.height))
        .map(|(width, height)| SourceSize::new(width, height))
        .ok_or(Error::Size {
            path: path.to_path_buf(),
            width: file.width,
            height: file.height,
        })?;
    let rate = FrameRate::new(file.frame_rate).map_err(|source| Error::Rate {
        path: path.to_path_buf(),
        source,
    })?;
    let damaged = file
        .damaged_spans
        .iter()
        .map(|&[first, last]| Span::new(first, last))
        .collect::<Result<_, _>>()
        .map_err(|source| Error::Span {
            path: path.to_path_buf(),
            source,
        })?;
    let half_window = NonZeroUsize::new(file.half_window).ok_or(Error::HalfWindow {
        path: path.to_path_buf(),
    })?;
    Ok(Video {
        path: file.video.clone(),
        size,
        rate,
        damaged,
        reported_frames: file.reported_frames,
        half_window,
    })
}

fn provenance(targets: TargetsFile, path: &Path) -> Result<Provenance, Error> {
    match targets {
        TargetsFile::Replayed(replayed) => Ok(Provenance::Replayed(replayed)),
        TargetsFile::Drawn {
            seed,
            classes,
            per_class,
            min_distance,
            track,
            stability,
        } => {
            let classes = classes
                .into_iter()
                .map(|class| {
                    let motion = class.motion.parse().map_err(|source| Error::Motion {
                        path: path.to_path_buf(),
                        source,
                    })?;
                    Class::new(class.amplitude, class.sigma, motion).map_err(|source| Error::Class {
                        path: path.to_path_buf(),
                        source,
                    })
                })
                .collect::<Result<_, _>>()?;
            let per_class = NonZeroUsize::new(per_class).ok_or(Error::PerClass {
                path: path.to_path_buf(),
            })?;
            Ok(Provenance::Drawn {
                sampling: Sampling {
                    seed,
                    classes,
                    per_class,
                    min_distance,
                },
                track,
                stability,
            })
        }
    }
}
