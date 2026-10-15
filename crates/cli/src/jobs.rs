use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use files::names::{RESULTS_FOLDER, composed};

#[cfg(test)]
mod tests;

const VIDEO_EXTENSIONS: [&str; 5] = ["mp4", "mov", "avi", "mkv", "m4v"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub video: PathBuf,
    pub dest: PathBuf,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot list the videos of {}", folder.display())]
    Folder {
        folder: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("several videos would write to {}", dest.display())]
    SharedFolder { dest: PathBuf },
}

pub fn collect_videos(inputs: &[PathBuf]) -> Result<Vec<PathBuf>, Error> {
    let mut videos = Vec::new();
    for input in inputs {
        if input.is_dir() {
            videos.extend(videos_of(input)?);
        } else {
            videos.push(input.clone());
        }
    }
    Ok(videos)
}

fn videos_of(folder: &Path) -> Result<Vec<PathBuf>, Error> {
    let failed = |source| Error::Folder {
        folder: folder.to_path_buf(),
        source,
    };
    let mut videos = Vec::new();
    for entry in fs::read_dir(folder).map_err(failed)? {
        let path = entry.map_err(failed)?.path();
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

pub fn plan_jobs(videos: &[PathBuf], out: Option<&Path>) -> Result<Vec<Job>, Error> {
    let roots: Vec<PathBuf> = videos.iter().map(|video| root(video, out)).collect();
    let keys: Vec<Homonym> = videos
        .iter()
        .zip(&roots)
        .map(|(video, root)| Homonym {
            root: folded(root.as_os_str()),
            stem: folded(video.file_stem().unwrap_or_default()),
        })
        .collect();
    let mut counts: BTreeMap<&Homonym, usize> = BTreeMap::new();
    for key in &keys {
        *counts.entry(key).or_default() += 1;
    }
    let jobs: Vec<Job> = videos
        .iter()
        .zip(roots)
        .zip(&keys)
        .map(|((video, root), key)| {
            let ambiguous = counts.get(key).is_some_and(|&count| count > 1);
            Job {
                video: video.clone(),
                dest: root.join(output_name(video, ambiguous, out.is_some())),
            }
        })
        .collect();
    let mut seen = BTreeSet::new();
    if let Some(shared) = jobs.iter().find(|job| !seen.insert(folded(job.dest.as_os_str()))) {
        return Err(Error::SharedFolder {
            dest: shared.dest.clone(),
        });
    }
    Ok(jobs)
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Homonym {
    root: Vec<u8>,
    stem: Vec<u8>,
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
