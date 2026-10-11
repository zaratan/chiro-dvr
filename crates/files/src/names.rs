use std::ffi::{OsStr, OsString};
use std::ops::Range;
use std::path::{Path, PathBuf};

use types::language::Language;
use types::track::Track;

#[cfg(test)]
mod tests;

const PARAMS: &str = "params.json";
const ZOOM_SUFFIX: &str = "_zoom";
const VIDEO_EXTENSION: &str = ".mp4";
const MIN_TRACK_DIGITS: usize = 2;
const SECONDS_PER_MINUTE: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Names {
    clips_dir: &'static str,
    track: &'static str,
    tracks_csv: &'static str,
    summary: &'static str,
    annotated: &'static str,
}

const FRENCH: Names = Names {
    clips_dir: "extraits",
    track: "piste",
    tracks_csv: "_pistes",
    summary: "_resume",
    annotated: "_annotee",
};

const ENGLISH: Names = Names {
    clips_dir: "clips",
    track: "track",
    tracks_csv: "_tracks",
    summary: "_summary",
    annotated: "_annotated",
};

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
        format!("{}{VIDEO_EXTENSION}", self.clip_stem(track, start_seconds, track_count))
    }

    pub fn zoomed_clip(self, track: &Track, start_seconds: f64, track_count: usize) -> String {
        format!(
            "{}{ZOOM_SUFFIX}{VIDEO_EXTENSION}",
            self.clip_stem(track, start_seconds, track_count)
        )
    }

    pub fn tracks_csv(self, dest: &Path, stem: &OsStr) -> PathBuf {
        dest.join(suffixed(stem, &format!("{}.csv", self.tracks_csv)))
    }

    pub fn summary_png(self, dest: &Path, stem: &OsStr, period_minutes: Option<Range<u32>>) -> PathBuf {
        let period = period_minutes.map_or_else(String::new, |minutes| {
            format!("_{:03}m-{:03}m", minutes.start, minutes.end)
        });
        dest.join(suffixed(stem, &format!("{}{period}.png", self.summary)))
    }

    pub fn annotated_video(self, dest: &Path, stem: &OsStr) -> PathBuf {
        dest.join(suffixed(stem, &format!("{}{VIDEO_EXTENSION}", self.annotated)))
    }

    fn clip_stem(self, track: &Track, start_seconds: f64, track_count: usize) -> String {
        let whole_seconds = start_seconds.max(0.0).floor() as u64;
        let (minutes, seconds) = (whole_seconds / SECONDS_PER_MINUTE, whole_seconds % SECONDS_PER_MINUTE);
        let width = MIN_TRACK_DIGITS.max(track_count.to_string().len());
        format!("{}_{:0width$}_{minutes}m{seconds:02}s", self.track, track.id())
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
