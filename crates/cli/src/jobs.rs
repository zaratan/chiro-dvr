use std::collections::{BTreeMap, BTreeSet};
use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

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

pub fn plan_jobs(videos: &[PathBuf], out: &Path) -> Result<Vec<Job>, Error> {
    let mut stems: BTreeMap<&OsStr, usize> = BTreeMap::new();
    for video in videos {
        *stems.entry(video.file_stem().unwrap_or_default()).or_default() += 1;
    }
    let jobs: Vec<Job> = videos
        .iter()
        .map(|video| {
            let ambiguous = stems
                .get(video.file_stem().unwrap_or_default())
                .is_some_and(|&count| count > 1);
            Job {
                video: video.clone(),
                dest: out.join(output_name(video, ambiguous)),
            }
        })
        .collect();
    let mut seen = BTreeSet::new();
    if let Some(shared) = jobs.iter().find(|job| !seen.insert(&job.dest)) {
        return Err(Error::SharedFolder {
            dest: shared.dest.clone(),
        });
    }
    Ok(jobs)
}

fn output_name(video: &Path, ambiguous: bool) -> OsString {
    let stem = video.file_stem().unwrap_or_default();
    if !ambiguous {
        return stem.to_os_string();
    }
    let mut name = video
        .parent()
        .and_then(Path::file_name)
        .unwrap_or_default()
        .to_os_string();
    name.push("_");
    name.push(stem);
    name.push("_");
    name.push(video.extension().unwrap_or_default().to_ascii_lowercase());
    name
}
