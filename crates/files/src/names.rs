use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::ops::Range;
use std::path::{Path, PathBuf};

use tempfile::{Builder, NamedTempFile, TempPath};
use types::language::Language;
use types::track::Track;
use unicode_normalization::UnicodeNormalization;

#[cfg(test)]
mod tests;

const PARAMS: &str = "params.json";
const FRENCH_PAGE: &str = "resultats.html";
const ENGLISH_PAGE: &str = "results.html";
pub const RESULTS_FOLDER: &str = "batdetect";
const ZOOM_SUFFIX: &str = "_zoom";
const VIDEO_EXTENSION: &str = ".mp4";
const TABLE_EXTENSION: &str = ".csv";
const IMAGE_EXTENSION: &str = ".png";
const PARTIAL_PREFIX: &str = ".";
const PARTIAL_SUFFIX: &str = ".partial";
const UNIQUE_SEPARATOR: &str = ".";
const UNIQUE_LENGTH: usize = 6;
const HIDDEN_PREFIX: &[u8] = b".";
const PERIOD_DIGITS: usize = 3;
const MIN_TRACK_DIGITS: usize = 2;
const SECONDS_PER_MINUTE: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Part {
    pub number: usize,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Names {
    clips_dir: &'static str,
    track: &'static str,
    tracks_csv: &'static str,
    summary: &'static str,
    part_of: &'static str,
    annotated: &'static str,
}

const FRENCH: Names = Names {
    clips_dir: "extraits",
    track: "piste",
    tracks_csv: "_pistes",
    summary: "_resume",
    part_of: "sur",
    annotated: "_annotee",
};

const ENGLISH: Names = Names {
    clips_dir: "clips",
    track: "track",
    tracks_csv: "_tracks",
    summary: "_summary",
    part_of: "of",
    annotated: "_annotated",
};

const LEGACY: Names = Names {
    clips_dir: "split",
    track: "",
    tracks_csv: ".tracks",
    summary: ".tracks",
    part_of: "",
    annotated: "_boxes",
};

const EVERY: [Names; 3] = [FRENCH, ENGLISH, LEGACY];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Output {
    ClipsFolder,
    TracksTable,
    Rendered,
    Partial,
}

impl Names {
    pub fn new(language: Language) -> Self {
        match language {
            Language::French => FRENCH,
            Language::English => ENGLISH,
        }
    }

    pub fn clips_dir(self, dest: &Path) -> PathBuf {
        dest.join(self.clips_dir)
    }

    pub fn clip(self, track: &Track, start_seconds: f64, track_count: usize) -> String {
        self.numbered_clip(track.id(), start_seconds, track_count)
    }

    pub fn zoomed_clip(self, track: &Track, start_seconds: f64, track_count: usize) -> String {
        self.numbered_zoomed_clip(track.id(), start_seconds, track_count)
    }

    pub fn numbered_clip(self, number: usize, start_seconds: f64, track_count: usize) -> String {
        format!(
            "{}{VIDEO_EXTENSION}",
            self.clip_stem(number, start_seconds, track_count)
        )
    }

    pub fn numbered_zoomed_clip(self, number: usize, start_seconds: f64, track_count: usize) -> String {
        format!(
            "{}{ZOOM_SUFFIX}{VIDEO_EXTENSION}",
            self.clip_stem(number, start_seconds, track_count)
        )
    }

    pub fn tracks_csv(self, dest: &Path, stem: &OsStr) -> PathBuf {
        dest.join(suffixed(stem, &format!("{}{TABLE_EXTENSION}", self.tracks_csv)))
    }

    pub fn summary_png(
        self,
        dest: &Path,
        stem: &OsStr,
        period_minutes: Option<Range<u32>>,
        part: Option<Part>,
    ) -> PathBuf {
        let period = period_minutes.map_or_else(String::new, period_suffix);
        let part = part.map_or_else(String::new, |part| self.part_suffix(part));
        dest.join(suffixed(
            stem,
            &format!("{}{period}{part}{IMAGE_EXTENSION}", self.summary),
        ))
    }

    pub fn part_suffix(self, Part { number, count }: Part) -> String {
        let width = count.to_string().len();
        format!("_{number:0width$}_{}_{count}", self.part_of)
    }

    pub fn annotated_video(self, dest: &Path, stem: &OsStr) -> PathBuf {
        dest.join(suffixed(stem, &format!("{}{VIDEO_EXTENSION}", self.annotated)))
    }

    fn clip_stem(self, number: usize, start_seconds: f64, track_count: usize) -> String {
        let whole_seconds = start_seconds.max(0.0).floor() as u64;
        let (minutes, seconds) = (whole_seconds / SECONDS_PER_MINUTE, whole_seconds % SECONDS_PER_MINUTE);
        let width = MIN_TRACK_DIGITS.max(track_count.to_string().len());
        format!("{}_{number:0width$}_{minutes}m{seconds:02}s", self.track)
    }

    fn output(self, name: &[u8], stem: &[u8]) -> Option<Output> {
        if name == self.clips_dir.as_bytes() {
            return Some(Output::ClipsFolder);
        }
        let rest = name.strip_prefix(stem)?;
        if rest.strip_prefix(self.tracks_csv.as_bytes()) == Some(TABLE_EXTENSION.as_bytes()) {
            Some(Output::TracksTable)
        } else if rest.strip_prefix(self.annotated.as_bytes()) == Some(VIDEO_EXTENSION.as_bytes())
            || self.is_summary(rest)
        {
            Some(Output::Rendered)
        } else {
            None
        }
    }

    fn is_summary(self, rest: &[u8]) -> bool {
        let Some(middle) = rest
            .strip_prefix(self.summary.as_bytes())
            .and_then(|middle| middle.strip_suffix(IMAGE_EXTENSION.as_bytes()))
        else {
            return false;
        };
        let after = after_period(middle).unwrap_or(middle);
        after.is_empty() || !self.part_of.is_empty() && self.after_part(after).is_some_and(<[u8]>::is_empty)
    }

    fn after_part(self, text: &[u8]) -> Option<&[u8]> {
        let text = after_digits(text.strip_prefix(b"_")?, 1)?;
        let text = text.strip_prefix(b"_")?.strip_prefix(self.part_of.as_bytes())?;
        after_digits(text.strip_prefix(b"_")?, 1)
    }
}

pub fn names_the_video(name: &[u8], stem: &[u8]) -> bool {
    previous_output(name, stem).is_some_and(|(_, output)| output != Output::ClipsFolder)
}

pub(crate) fn previous_output(name: &[u8], stem: &[u8]) -> Option<(Names, Output)> {
    let finished = |name: &[u8]| {
        EVERY
            .into_iter()
            .find_map(|names| Some((names, names.output(name, stem)?)))
    };
    let partial = finished_names_of_partial(name)
        .find_map(|name| finished(name).filter(|(_, output)| *output != Output::ClipsFolder))
        .map(|(names, _)| (names, Output::Partial));
    partial.or_else(|| finished(name))
}

pub(crate) fn is_partial_params(name: &[u8]) -> bool {
    finished_names_of_partial(name).any(|finished| finished == PARAMS.as_bytes())
}

fn finished_names_of_partial(name: &[u8]) -> impl Iterator<Item = &[u8]> {
    let fixed = name
        .strip_prefix(PARTIAL_PREFIX.as_bytes())
        .and_then(|name| name.strip_suffix(PARTIAL_SUFFIX.as_bytes()));
    let unique = fixed.and_then(without_unique_part);
    fixed.into_iter().chain(unique)
}

fn without_unique_part(name: &[u8]) -> Option<&[u8]> {
    let (finished, unique) = name.split_at_checked(name.len().checked_sub(UNIQUE_LENGTH)?)?;
    let finished = finished.strip_suffix(UNIQUE_SEPARATOR.as_bytes())?;
    unique.iter().all(u8::is_ascii_alphanumeric).then_some(finished)
}

fn after_period(text: &[u8]) -> Option<&[u8]> {
    let text = after_digits(text.strip_prefix(b"_")?, PERIOD_DIGITS)?;
    after_digits(text.strip_prefix(b"m-")?, PERIOD_DIGITS)?.strip_prefix(b"m")
}

fn after_digits(text: &[u8], at_least: usize) -> Option<&[u8]> {
    let count = text.iter().take_while(|byte| byte.is_ascii_digit()).count();
    (count >= at_least).then(|| &text[count..])
}

pub fn composed(name: &str) -> String {
    name.nfc().collect()
}

pub fn is_hidden(name: &OsStr) -> bool {
    name.as_encoded_bytes().starts_with(HIDDEN_PREFIX)
}

pub fn partial_file(path: &Path) -> io::Result<TempPath> {
    let (Some(name), Some(folder)) = (path.file_name(), path.parent()) else {
        return Err(io::Error::from(io::ErrorKind::InvalidInput));
    };
    let mut prefix = OsString::from(PARTIAL_PREFIX);
    prefix.push(name);
    prefix.push(UNIQUE_SEPARATOR);
    Builder::new()
        .prefix(&prefix)
        .suffix(PARTIAL_SUFFIX)
        .rand_bytes(UNIQUE_LENGTH)
        .make_in(folder, |path| File::create_new(path))
        .map(NamedTempFile::into_temp_path)
}

pub fn period_suffix(minutes: Range<u32>) -> String {
    format!("_{:03}m-{:03}m", minutes.start, minutes.end)
}

pub const fn results_page(language: Language) -> &'static str {
    match language {
        Language::French => FRENCH_PAGE,
        Language::English => ENGLISH_PAGE,
    }
}

pub fn params_json(dest: &Path) -> PathBuf {
    dest.join(PARAMS)
}

fn suffixed(stem: &OsStr, suffix: &str) -> OsString {
    let mut name = stem.to_os_string();
    name.push(suffix);
    name
}
