use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use files::names::{RESULTS_FOLDER, composed, is_hidden};

use crate::access::Access;
use crate::failure::Cause;

#[cfg(test)]
mod tests;

const CURRENT_FOLDER: &str = ".";
const INPUT_MISSING: &str = "input-missing";
pub const UNREACHABLE: &str = "unreachable";
const NOT_A_VIDEO_NAME: &str = "not-a-video-name";
pub const NO_VIDEO: &str = "no-video";
pub const SHARED_FOLDER: &str = "shared-folder";
pub const NOT_A_VIDEO: &str = "not-a-video";
pub const UNOPENED: &str = "unopened";
const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "mov", "avi", "mkv", "m4v"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub video: PathBuf,
    pub dest: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reason {
    Missing,
    NotAVideo,
    NoVideo,
    Unreadable(Access),
    SharedFolder { videos: Vec<PathBuf> },
    Unopened { cause: Cause, detail: String },
}

impl Reason {
    pub const fn key(&self) -> &'static str {
        match self {
            Self::Missing | Self::Unreadable(Access::Missing) => "missing",
            Self::Unreadable(Access::Refused) => "refused",
            Self::Unreadable(Access::NotAFolder) => "not-a-folder",
            Self::Unreadable(Access::ReadOnly | Access::DiskFull | Access::Unavailable) => "unavailable",
            Self::NotAVideo => NOT_A_VIDEO,
            Self::NoVideo => NO_VIDEO,
            Self::SharedFolder { .. } => SHARED_FOLDER,
            Self::Unopened { .. } => UNOPENED,
        }
    }

    pub const fn todo(&self) -> &'static str {
        match self {
            Self::Missing | Self::Unreadable(Access::Missing | Access::NotAFolder) => INPUT_MISSING,
            Self::Unreadable(Access::Refused) => Cause::ReadRefused.key(),
            Self::Unreadable(Access::ReadOnly | Access::DiskFull | Access::Unavailable) => UNREACHABLE,
            Self::NotAVideo => NOT_A_VIDEO_NAME,
            Self::NoVideo => NO_VIDEO,
            Self::SharedFolder { .. } => SHARED_FOLDER,
            Self::Unopened { cause, .. } => cause.key(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    pub path: PathBuf,
    pub reason: Reason,
}

impl Refused {
    fn names(&self, video: &Path) -> bool {
        matches!(&self.reason, Reason::SharedFolder { videos } if videos.iter().any(|shared| shared == video))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("no video found")]
    NoVideo { folders: Vec<PathBuf> },
    #[error("{} inputs are refused", refused.len())]
    Refused { refused: Vec<Refused>, valid: Vec<Job> },
}

pub const fn video_extensions() -> &'static [&'static str] {
    &VIDEO_EXTENSIONS
}

pub fn plan(inputs: &[PathBuf], out: Option<&Path>) -> Result<Vec<Job>, Error> {
    let Collected { videos, mut refused } = collect_videos(inputs);
    let (jobs, shared) = plan_jobs(&videos, out);
    let jobs: Vec<Job> = jobs
        .into_iter()
        .filter(|job| !shared.iter().any(|refusal| refusal.names(&job.video)))
        .collect();
    refused.extend(shared);
    if jobs.is_empty() && refused.iter().all(|refusal| refusal.reason == Reason::NoVideo) {
        return Err(Error::NoVideo {
            folders: refused.into_iter().map(|refusal| refusal.path).collect(),
        });
    }
    if refused.is_empty() {
        Ok(jobs)
    } else {
        Err(Error::Refused { refused, valid: jobs })
    }
}

#[derive(Debug, Default)]
struct Collected {
    videos: Vec<PathBuf>,
    refused: Vec<Refused>,
}

fn collect_videos(inputs: &[PathBuf]) -> Collected {
    let mut collected = Collected::default();
    for input in inputs {
        let reason = match fs::metadata(input) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Reason::Missing,
            Err(error) => Reason::Unreadable(Access::from(error.kind())),
            Ok(found) if found.is_dir() => match videos_of(input) {
                Ok(videos) if videos.is_empty() => Reason::NoVideo,
                Ok(videos) => {
                    collected.videos.extend(videos);
                    continue;
                }
                Err(error) => Reason::Unreadable(Access::from(error.kind())),
            },
            Ok(_) if is_video(input) => {
                collected.videos.push(input.clone());
                continue;
            }
            Ok(_) => Reason::NotAVideo,
        };
        collected.refused.push(Refused {
            path: input.clone(),
            reason,
        });
    }
    let mut seen = BTreeSet::new();
    collected.videos.retain(|video| seen.insert(identity(video)));
    let mut seen = BTreeSet::new();
    collected.refused.retain(|refusal| seen.insert(identity(&refusal.path)));
    collected
}

fn videos_of(folder: &Path) -> io::Result<Vec<PathBuf>> {
    let mut videos = Vec::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        if is_hidden(&entry.file_name()) {
            continue;
        }
        let path = entry.path();
        if path.is_file() && is_video(&path) {
            videos.push(path);
        }
    }
    videos.sort();
    Ok(videos)
}

fn is_video(path: &Path) -> bool {
    path.extension().is_some_and(|extension| {
        VIDEO_EXTENSIONS
            .iter()
            .any(|known| extension.eq_ignore_ascii_case(known))
    })
}

fn plan_jobs(videos: &[PathBuf], out: Option<&Path>) -> (Vec<Job>, Vec<Refused>) {
    let real_roots: Vec<PathBuf> = videos.iter().map(|video| real_root(video, out)).collect();
    let mut written: BTreeMap<Vec<u8>, PathBuf> = BTreeMap::new();
    let roots: Vec<PathBuf> = videos
        .iter()
        .zip(&real_roots)
        .map(|(video, real)| {
            written
                .entry(folded(real.as_os_str()))
                .or_insert_with(|| root(video, out))
                .clone()
        })
        .collect();
    let keys: Vec<Homonym> = videos
        .iter()
        .zip(&real_roots)
        .map(|(video, real)| Homonym {
            root: folded(real.as_os_str()),
            stem: folded(video.file_stem().unwrap_or_default()),
        })
        .collect();
    let mut counts: BTreeMap<&Homonym, usize> = BTreeMap::new();
    for key in &keys {
        *counts.entry(key).or_default() += 1;
    }
    let mut shared: BTreeMap<Vec<u8>, (usize, PathBuf, Vec<PathBuf>)> = BTreeMap::new();
    let mut jobs = Vec::with_capacity(videos.len());
    for ((video, (root, real)), key) in videos.iter().zip(roots.iter().zip(&real_roots)).zip(&keys) {
        let ambiguous = counts.get(key).is_some_and(|&count| count > 1);
        let name = output_name(video, ambiguous, out.is_some());
        let dest = root.join(&name);
        let rank = shared.len();
        shared
            .entry(folded(real.join(&name).as_os_str()))
            .or_insert_with(|| (rank, dest.clone(), Vec::new()))
            .2
            .push(video.clone());
        jobs.push(Job {
            video: video.clone(),
            dest,
        });
    }
    let mut groups: Vec<(usize, PathBuf, Vec<PathBuf>)> =
        shared.into_values().filter(|(_, _, videos)| videos.len() > 1).collect();
    groups.sort_by_key(|(rank, _, _)| *rank);
    let shared = groups
        .into_iter()
        .map(|(_, dest, videos)| Refused {
            path: dest,
            reason: Reason::SharedFolder { videos },
        })
        .collect();
    (jobs, shared)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Homonym {
    root: Vec<u8>,
    stem: Vec<u8>,
}

fn real_root(video: &Path, out: Option<&Path>) -> PathBuf {
    out.map_or_else(
        || real(video.parent().unwrap_or(Path::new(""))).join(RESULTS_FOLDER),
        real,
    )
}

fn identity(video: &Path) -> PathBuf {
    fs::canonicalize(video).unwrap_or_else(|_| match video.file_name() {
        Some(name) => real(video.parent().unwrap_or(Path::new(""))).join(name),
        None => video.to_path_buf(),
    })
}

fn real(path: &Path) -> PathBuf {
    let path = if path.as_os_str().is_empty() {
        Path::new(CURRENT_FOLDER)
    } else {
        path
    };
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn root(video: &Path, out: Option<&Path>) -> PathBuf {
    out.map_or_else(|| video.with_file_name(RESULTS_FOLDER), Path::to_path_buf)
}

fn folded(name: &OsStr) -> Vec<u8> {
    name.to_str().map_or_else(
        || name.as_encoded_bytes().to_vec(),
        |text| composed(text).to_lowercase().into_bytes(),
    )
}

fn output_name(video: &Path, ambiguous: bool, with_parent: bool) -> OsString {
    let stem = video.file_stem().unwrap_or_default();
    if !ambiguous {
        return stem.to_os_string();
    }
    let mut name = OsString::new();
    if with_parent {
        name.push(video.parent().and_then(Path::file_name).unwrap_or_default());
        name.push("_");
    }
    name.push(stem);
    name.push("_");
    name.push(video.extension().unwrap_or_default().to_ascii_lowercase());
    name
}
