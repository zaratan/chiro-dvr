#[derive(Debug, Clone, PartialEq)]
pub struct Page {
    pub title: String,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Entry {
    Video(Video),
    Failed(Failed),
    Unfinished { name: String },
    Older { name: String },
    Waiting { name: String },
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
pub struct Summary {
    pub file: String,
    pub width: u32,
    pub height: u32,
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
    pub verdict: Option<String>,
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

impl Page {
    pub fn tracks(&self) -> usize {
        self.entries
            .iter()
            .map(|entry| match entry {
                Entry::Video(video) => video.tracks.len(),
                Entry::Failed(_) | Entry::Unfinished { .. } | Entry::Older { .. } | Entry::Waiting { .. } => 0,
            })
            .sum()
    }
}
