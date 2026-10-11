use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use types::timeline::Timeline;
use types::track::Track;

use crate::error::Error;
use crate::time::{clock, frame_duration, frame_seconds};

#[cfg(test)]
mod tests;

const HEADER: &str =
    "id,start,end,start_s,duration_s,hits,chord_px,path_px,speed_px_s,max_area_px,peak_amplitude,filled_frames";
const LINE_END: &str = "\r\n";

pub fn write_tracks_csv(path: &Path, tracks: &[Track], timeline: &Timeline) -> Result<(), Error> {
    let failed = |source| Error::Write {
        path: path.to_path_buf(),
        source,
    };
    let mut file = BufWriter::new(File::create(path).map_err(failed)?);
    write!(file, "{HEADER}{LINE_END}").map_err(failed)?;
    for track in tracks {
        write!(file, "{}{LINE_END}", row(track, timeline)).map_err(failed)?;
    }
    file.flush().map_err(failed)
}

fn row(track: &Track, timeline: &Timeline) -> String {
    let (first, last) = (track.first(), track.last());
    let start = frame_seconds(timeline, first.frame);
    let end = frame_seconds(timeline, last.frame);
    let duration = (end - start).max(frame_duration(timeline));
    let path = track.path_length();
    let spanned = last.frame.get() - first.frame.get() + 1;
    [
        track.id().to_string(),
        clock(start),
        clock(end),
        decimal(start, 2),
        decimal(duration, 2),
        track.points().len().to_string(),
        decimal(track.chord(), 1),
        decimal(path, 1),
        decimal(path / duration, 1),
        (track.max_area().max(0.0).round() as u64).to_string(),
        decimal(track.peak_amplitude(), 1),
        (spanned - track.points().len()).to_string(),
    ]
    .join(",")
}

fn decimal(value: f64, places: i32) -> String {
    let scale = 10_f64.powi(places);
    let rounded = (value * scale).round() / scale + 0.0;
    let text = rounded.to_string();
    if rounded.is_finite() && !text.contains('.') {
        text + ".0"
    } else {
        text
    }
}
