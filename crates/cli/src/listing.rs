use std::path::{Component, Path};

use files::params::{Trace, TrackEntry};
use files::time::track_seconds;
use render::clips::{Clip, Outputs};
use summary::Painted;
use types::timeline::Timeline;
use types::track::Track;

use crate::overview::Drawing;

#[cfg(test)]
mod tests;

const SEPARATOR: &str = "/";
const POINT_SEPARATOR: &str = " ";

pub(crate) fn listed(
    dest: &Path,
    tracks: &[Track],
    timeline: &Timeline,
    painted: &[Drawing],
    outputs: Option<&Outputs>,
) -> Vec<TrackEntry> {
    tracks
        .iter()
        .enumerate()
        .map(|(index, track)| {
            let [start_s, duration_s] = track_seconds(track, timeline);
            let written = |clips: &[Clip]| {
                clips
                    .iter()
                    .find(|clip| clip.track == index)
                    .and_then(|clip| relative(dest, &clip.path))
            };
            let drawing = painted.iter().find(|one| one.painted.track == track.id());
            TrackEntry {
                id: track.id(),
                start_s,
                duration_s,
                color: drawing.map(|one| {
                    let color = one.painted.color;
                    [color.red, color.green, color.blue]
                }),
                summary: drawing.and_then(|one| relative(dest, &one.page)),
                clip: outputs.and_then(|outputs| written(&outputs.clips)),
                zoom: outputs.and_then(|outputs| written(&outputs.zooms)),
                start_xy: Some(pixel([track.first().x, track.first().y])),
                trace: drawing.map(|one| trace(&one.painted)),
            }
        })
        .collect()
}

pub(crate) fn relative(dest: &Path, path: &Path) -> Option<String> {
    let parts: Vec<String> = path
        .strip_prefix(dest)
        .ok()?
        .components()
        .map(|part| match part {
            Component::Normal(name) => Some(name.to_string_lossy().into_owned()),
            _ => None,
        })
        .collect::<Option<_>>()?;
    Some(parts.join(SEPARATOR))
}

fn trace(painted: &Painted) -> Trace {
    Trace {
        points: points_text(&painted.path),
        head: painted.head.map(|head| points_text(&head)),
        dot: pixel(painted.dot),
        anchor: painted.anchor.map(pixel),
    }
}

fn points_text(spots: &[[f64; 2]]) -> String {
    let mut pixels: Vec<[i64; 2]> = spots.iter().map(|&spot| pixel(spot)).collect();
    pixels.dedup();
    pixels
        .iter()
        .map(|[x, y]| format!("{x},{y}"))
        .collect::<Vec<_>>()
        .join(POINT_SEPARATOR)
}

fn pixel([x, y]: [f64; 2]) -> [i64; 2] {
    [x.round() as i64, y.round() as i64]
}
