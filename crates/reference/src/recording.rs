use std::path::Path;

use types::rate::FrameRate;
use types::timeline::Timeline;
use types::track::Track;

use crate::error::Error;
use crate::frames::Frames;
use crate::table::{self, TrackRow};
use crate::tracks;
use crate::video::VideoSummary;

#[cfg(test)]
mod tests;

pub const SUMMARY_FILE: &str = "video.json";

#[derive(Debug, Clone, PartialEq)]
pub struct Recording {
    pub summary: VideoSummary,
    pub frames: Frames,
    pub tracks: Vec<Track>,
    pub rows: Option<Vec<TrackRow>>,
}

impl Recording {
    pub fn read(directory: &Path) -> Result<Self, Error> {
        let summary = VideoSummary::read(&directory.join(SUMMARY_FILE))?;
        let frames = Frames::read(&directory.join(&summary.files.frames))?;
        let tracks = tracks::read(&directory.join(&summary.files.tracks))?;
        let rows = summary
            .files
            .csv
            .as_ref()
            .map(|csv| table::read(&directory.join(csv)))
            .transpose()?;
        Ok(Self {
            summary,
            frames,
            tracks,
            rows,
        })
    }

    pub fn write(&self, directory: &Path) -> Result<(), Error> {
        let table = match (&self.summary.files.csv, &self.rows) {
            (Some(csv), Some(rows)) => Some((csv, rows)),
            (None, None) => None,
            (Some(_), None) | (None, Some(_)) => return Err(Error::TableWithoutFile),
        };
        self.summary.write(&directory.join(SUMMARY_FILE))?;
        self.frames.write(&directory.join(&self.summary.files.frames))?;
        tracks::write(&directory.join(&self.summary.files.tracks), &self.tracks)?;
        match table {
            Some((csv, rows)) => table::write(&directory.join(csv), rows),
            None => Ok(()),
        }
    }

    pub fn timeline(&self) -> Result<Timeline, Error> {
        let stream = &self.summary.stream;
        let rate = FrameRate::new(stream.avg_frame_rate)?;
        Ok(Timeline::new(self.frames.pts(), stream.time_base, rate)?)
    }
}
