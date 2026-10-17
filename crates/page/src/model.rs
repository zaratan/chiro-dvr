#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub title: String,
    pub entries: Vec<Entry>,
    pub refused: Option<Refusals>,
    pub verdicts: Option<VerdictsFile>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerdictsFile {
    pub name: String,
    pub modified: Option<Date>,
    pub candidates: usize,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Refusals {
    pub date: Option<Date>,
    pub inputs: Vec<RefusedInput>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefusedInput {
    pub path: String,
    pub reason: String,
    pub cause: Option<String>,
    pub todo: String,
    pub videos: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Video(Video),
    Failed(Failed),
    Unfinished { name: String },
    Older { name: String },
    Waiting { name: String },
    NotStarted { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: i16,
    pub month: i8,
    pub day: i8,
    pub hour: i8,
    pub minute: i8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Video {
    pub name: String,
    pub folder: String,
    pub date: Option<Date>,
    pub length_s: f64,
    pub not_analysed_s: f64,
    pub open: bool,
    pub too_many: Option<TooMany>,
    pub background: Option<Background>,
    pub summaries: Vec<Summary>,
    pub tracks: Vec<Track>,
    pub table: Option<String>,
    pub annotated: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooMany {
    pub count: usize,
    pub limit: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Background {
    pub file: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Summary {
    pub file: String,
    pub width: u32,
    pub height: u32,
    pub layout: Option<Layout>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Layout {
    pub window_s: [f64; 2],
    pub ignored_s: Vec<[f64; 2]>,
    pub damaged_s: Vec<[f64; 2]>,
    pub other_starts_s: Vec<f64>,
    pub radius: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Track {
    pub id: usize,
    pub start_s: f64,
    pub duration_s: f64,
    pub color: Option<[u8; 3]>,
    pub summary: Option<String>,
    pub clip: Option<String>,
    pub zoom: Option<String>,
    pub start_xy: Option<[i64; 2]>,
    pub trace: Option<Trace>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trace {
    pub points: String,
    pub head: Option<String>,
    pub dot: [i64; 2],
    pub anchor: Option<[i64; 2]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Failed {
    pub name: String,
    pub date: Option<Date>,
    pub cause: String,
    pub place: String,
    pub folder: String,
    pub detail: Option<String>,
    pub previous: Option<Video>,
}

impl Video {
    pub(crate) fn viewable(&self) -> bool {
        !self.summaries.is_empty()
            || self
                .tracks
                .iter()
                .any(|track| track.clip.is_some() || track.zoom.is_some())
    }
}

impl Page {
    pub fn tracks(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| match entry {
                Entry::Video(video) => video.tracks.len(),
                Entry::Failed(_)
                | Entry::Unfinished { .. }
                | Entry::Older { .. }
                | Entry::Waiting { .. }
                | Entry::NotStarted { .. } => 0,
            })
            .sum()
    }
}
