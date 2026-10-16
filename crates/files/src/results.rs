use std::collections::BTreeMap;
use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use types::language::Language;

use crate::failures::{FailureRecord, read_failures};
use crate::names::{Names, names_the_video, params_json};
use crate::params::{Finish, TrackEntry};
use crate::picture::png_size;
use crate::table::Headings;
use crate::verdicts::read_verdicts;

#[cfg(test)]
mod tests;

const HIDDEN_PREFIX: &[u8] = b".";
const FRENCH: &str = "fr";

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Written {
    pub video: PathBuf,
    pub state: Finish,
    pub length_s: f64,
    pub not_analysed_s: f64,
    pub language: String,
    pub date: String,
    pub tracks: Vec<TrackEntry>,
    #[serde(rename = "RenderConfig")]
    pub render: Limits,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct Limits {
    pub max_tracks: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    pub file: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Finished {
    pub written: Written,
    pub summaries: Vec<Image>,
    pub verdicts: Vec<Option<String>>,
    pub table: Option<String>,
    pub annotated: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Found {
    Finished(Box<Finished>),
    Older,
    Unfinished,
    Absent,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Folder {
    pub name: OsString,
    pub found: Found,
    pub failure: Option<FailureRecord>,
}

pub fn read_results(root: &Path, headings: &dyn Headings) -> Vec<Folder> {
    let mut folders: BTreeMap<OsString, Folder> = BTreeMap::new();
    if let Ok(entries) = fs::read_dir(root) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let path = entry.path();
            if name.as_encoded_bytes().starts_with(HIDDEN_PREFIX) || !path.is_dir() {
                continue;
            }
            if let Some(found) = found(&path, &name, headings) {
                folders.insert(
                    name.clone(),
                    Folder {
                        name,
                        found,
                        failure: None,
                    },
                );
            }
        }
    }
    for (name, record) in read_failures(root) {
        folders
            .entry(name.clone())
            .or_insert_with(|| Folder {
                name,
                found: Found::Absent,
                failure: None,
            })
            .failure = Some(record);
    }
    folders.into_values().collect()
}

fn found(dest: &Path, name: &OsStr, headings: &dyn Headings) -> Option<Found> {
    let params = fs::read(params_json(dest)).ok();
    if let Some(written) = params
        .as_deref()
        .and_then(|bytes| serde_json::from_slice::<Written>(bytes).ok())
    {
        return Some(Found::Finished(Box::new(finished(dest, written, headings))));
    }
    let our_params = params
        .as_deref()
        .and_then(|bytes| serde_json::from_slice::<Ours>(bytes).ok())
        .is_some();
    if our_params {
        Some(Found::Older)
    } else {
        holds_our_outputs(dest, name).then_some(Found::Unfinished)
    }
}

#[derive(Deserialize)]
struct Ours {
    #[serde(rename = "video")]
    _video: PathBuf,
    #[serde(rename = "version")]
    _version: String,
}

fn holds_our_outputs(dest: &Path, name: &OsStr) -> bool {
    fs::read_dir(dest).is_ok_and(|entries| {
        entries
            .flatten()
            .any(|entry| names_the_video(entry.file_name().as_encoded_bytes(), name.as_encoded_bytes()))
    })
}

fn finished(dest: &Path, written: Written, headings: &dyn Headings) -> Finished {
    let language = if written.language == FRENCH {
        Language::French
    } else {
        Language::English
    };
    let names = Names::new(language);
    let stem = written.video.file_stem().unwrap_or_default().to_os_string();
    let present = |path: PathBuf| path.is_file().then(|| file_name(&path));
    let table_path = names.tracks_csv(dest, &stem);
    let verdicts = read_verdicts(&table_path, &written.tracks, headings);
    let mut files: Vec<String> = Vec::new();
    for file in written.tracks.iter().filter_map(|track| track.summary.clone()) {
        if !files.contains(&file) {
            files.push(file);
        }
    }
    if files.is_empty() {
        files.extend(present(names.summary_png(dest, &stem, None, None)));
    }
    let summaries = files
        .into_iter()
        .filter_map(|file| {
            let (width, height) = png_size(&dest.join(&file))?;
            Some(Image { file, width, height })
        })
        .collect();
    Finished {
        summaries,
        verdicts,
        table: present(table_path),
        annotated: present(names.annotated_video(dest, &stem)),
        written,
    }
}

fn file_name(path: &Path) -> String {
    path.file_name().unwrap_or_default().to_string_lossy().into_owned()
}
