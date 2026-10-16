use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use files::error::Error;
use files::failures::FailureRecord;
use files::names::partial_path;
use files::names::{RESULTS_FOLDER, composed, results_page};
use files::params::Finish;
use files::refusals::{RefusalRecord, read_refusals};
use files::results::{Finished, Found, read_results};
use files::table::Headings;
use jiff::Timestamp;
use jiff::tz::TimeZone;
use page::labels::Labels;
use page::model::{Date, Entry, Failed, Page, Refusals, RefusedInput, Summary, TooMany, Track, Video};
use tempfile::TempPath;
use types::language::Language;

use crate::failure::REFERS_TO_DETAIL;
use crate::jobs::Job;
use crate::refusals::VIDEOS_SEPARATOR;

#[cfg(test)]
mod tests;

const LANGUAGES: [Language; 2] = [Language::French, Language::English];
const MARKER_WINDOW: usize = 1024;

#[derive(Debug, Clone, Copy)]
pub struct Wording<'a> {
    pub labels: &'a dyn Labels,
    pub headings: &'a dyn Headings,
    pub language: Language,
}

pub fn roots(jobs: &[Job]) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    for root in jobs.iter().map(root_of) {
        if !roots.contains(&root) {
            roots.push(root);
        }
    }
    roots
}

pub fn root_of(job: &Job) -> PathBuf {
    job.dest.parent().unwrap_or(&job.dest).to_path_buf()
}

pub fn write_page(root: &Path, jobs: &[Job], finished: usize, wording: Wording<'_>) -> Result<(), Error> {
    if !root.is_dir() {
        return Ok(());
    }
    let page = page_of(root, jobs, finished, wording.headings);
    let html = page::render(&page, wording.labels);
    let target = root.join(results_page(wording.language));
    replace(&target, &html)?;
    for other in LANGUAGES.into_iter().filter(|&language| language != wording.language) {
        let path = root.join(results_page(other));
        if ours(&path) {
            fs::remove_file(&path).map_err(|source| Error::Remove { path, source })?;
        }
    }
    Ok(())
}

pub fn page_to_open(root: &Path, language: Language) -> Option<PathBuf> {
    let page = root.join(results_page(language));
    ours(&page).then_some(page)
}

fn replace(target: &Path, html: &str) -> Result<(), Error> {
    if target.exists() && !ours(target) {
        return Err(Error::Taken {
            path: target.to_path_buf(),
        });
    }
    let failed = |source| Error::Write {
        path: target.to_path_buf(),
        source,
    };
    let partial = TempPath::try_from_path(partial_path(target)).map_err(failed)?;
    File::create(&partial)
        .and_then(|mut file| file.write_all(html.as_bytes()))
        .map_err(failed)?;
    partial.persist(target).map_err(|failure| failed(failure.error))
}

fn ours(path: &Path) -> bool {
    let mut head = Vec::with_capacity(MARKER_WINDOW);
    File::open(path)
        .and_then(|file| file.take(MARKER_WINDOW as u64).read_to_end(&mut head))
        .is_ok_and(|_| page::is_ours(&head))
}

fn page_of(root: &Path, jobs: &[Job], finished: usize, headings: &dyn Headings) -> Page {
    let in_batch: Vec<(usize, &Job)> = jobs
        .iter()
        .enumerate()
        .filter(|(_, job)| root_of(job) == root)
        .collect();
    let in_batch_names: Vec<String> = in_batch
        .iter()
        .map(|(_, job)| key(job.dest.file_name().unwrap_or_default()))
        .collect();
    let mut entries: BTreeMap<String, Entry> = BTreeMap::new();
    for folder in read_results(root, headings) {
        let open = in_batch_names.contains(&key(&folder.name));
        let entry = match (folder.found, folder.failure) {
            (found, Some(record)) => Entry::Failed(failed(root, &folder.name, record, found)),
            (Found::Finished(done), None) => Entry::Video(video(&folder.name, &done, open)),
            (Found::Older, None) => Entry::Older {
                name: shown(&folder.name),
            },
            (Found::Unfinished | Found::Absent, None) => Entry::Unfinished {
                name: shown(&folder.name),
            },
        };
        entries.insert(key(&folder.name), entry);
    }
    for &(rank, job) in &in_batch {
        if rank >= finished {
            entries.insert(
                key(job.dest.file_name().unwrap_or_default()),
                Entry::Waiting {
                    name: shown(job.video.file_stem().unwrap_or_default()),
                },
            );
        }
    }
    Page {
        title: title(root),
        entries: entries.into_values().collect(),
        refused: read_refusals(root).map(refusals),
    }
}

fn refusals(record: RefusalRecord) -> Refusals {
    Refusals {
        date: date(&record.date),
        inputs: record
            .inputs
            .into_iter()
            .map(|input| RefusedInput {
                path: input.path,
                reason: input.reason,
                cause: input.cause,
                todo: input.todo,
                videos: input.videos.join(VIDEOS_SEPARATOR),
                detail: input.detail,
            })
            .collect(),
    }
}

fn failed(root: &Path, folder: &OsStr, record: FailureRecord, found: Found) -> Failed {
    let previous = match found {
        Found::Finished(done) => Some(video(folder, &done, false)),
        Found::Older | Found::Unfinished | Found::Absent => None,
    };
    let refers_to_detail = REFERS_TO_DETAIL.contains(&record.cause.as_str());
    Failed {
        name: shown(Path::new(&record.video).file_stem().unwrap_or_default()),
        date: date(&record.date),
        cause: record.cause,
        place: record.place,
        folder: shown(root.join(folder)),
        detail: record.detail.filter(|_| refers_to_detail),
        previous,
    }
}

fn video(folder: &OsStr, done: &Finished, open: bool) -> Video {
    let written = &done.written;
    let tracks = written
        .tracks
        .iter()
        .zip(&done.verdicts)
        .map(|(entry, verdict)| Track {
            id: entry.id,
            start_s: entry.start_s,
            duration_s: entry.duration_s,
            color: entry.color,
            summary: entry.summary.clone(),
            clip: entry.clip.clone(),
            zoom: entry.zoom.clone(),
            verdict: verdict.clone(),
        })
        .collect();
    Video {
        name: shown(written.video.file_stem().unwrap_or_default()),
        folder: shown(folder),
        date: date(&written.date),
        length_s: written.length_s,
        not_analysed_s: written.not_analysed_s,
        open,
        too_many: (written.state == Finish::TooManyTracks).then_some(TooMany {
            count: written.tracks.len(),
            limit: written.render.max_tracks,
        }),
        summaries: done
            .summaries
            .iter()
            .map(|image| Summary {
                file: image.file.clone(),
                width: image.width,
                height: image.height,
            })
            .collect(),
        tracks,
        table: done.table.clone(),
        annotated: done.annotated.clone(),
    }
}

fn date(written: &str) -> Option<Date> {
    let local = written.parse::<Timestamp>().ok()?.to_zoned(TimeZone::system());
    Some(Date {
        year: local.year(),
        month: local.month(),
        day: local.day(),
        hour: local.hour(),
        minute: local.minute(),
    })
}

fn title(root: &Path) -> String {
    let root = std::path::absolute(root).unwrap_or_else(|_| root.to_path_buf());
    let named = match root.file_name() {
        Some(name) if name == RESULTS_FOLDER => root.parent().and_then(Path::file_name),
        other => other,
    };
    shown(named.unwrap_or(root.as_os_str()))
}

fn key(name: &OsStr) -> String {
    composed(&name.to_string_lossy())
}

fn shown(path: impl AsRef<Path>) -> String {
    path.as_ref().to_string_lossy().into_owned()
}
