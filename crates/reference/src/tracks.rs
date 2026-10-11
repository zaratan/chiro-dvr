use std::path::Path;

use serde::{Deserialize, Serialize};
use types::frame::FrameIndex;
use types::track::Track;

use crate::error::Error;
use crate::lines;
use crate::measures::Measures;

#[cfg(test)]
mod tests;

#[derive(Serialize, Deserialize)]
struct TrackLine {
    id: usize,
    points: Vec<PointLine>,
}

#[derive(Serialize, Deserialize)]
struct PointLine {
    frame: FrameIndex,
    #[serde(flatten)]
    measures: Measures,
}

pub fn read(path: &Path) -> Result<Vec<Track>, Error> {
    let lines: Vec<TrackLine> = lines::read(path)?;
    lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            let points = line
                .points
                .into_iter()
                .map(|point| point.measures.at(point.frame))
                .collect();
            Track::new(line.id, points).map_err(|source| Error::Track {
                path: path.to_path_buf(),
                line: index + 1,
                source,
            })
        })
        .collect()
}

pub fn write(path: &Path, tracks: &[Track]) -> Result<(), Error> {
    let lines = tracks.iter().map(|track| TrackLine {
        id: track.id(),
        points: track
            .points()
            .iter()
            .map(|point| PointLine {
                frame: point.frame,
                measures: Measures::of(point),
            })
            .collect(),
    });
    lines::write(path, lines)
}
