use std::path::{Component, Path};

use files::params::TrackEntry;
use files::time::track_seconds;
use render::clips::{Clip, Outputs};
use summary::Painted;
use types::timeline::Timeline;
use types::track::Track;

#[cfg(test)]
mod tests;

const SEPARATOR: &str = "/";

pub(crate) fn listed(
    dest: &Path,
    tracks: &[Track],
    timeline: &Timeline,
    painted: &[Painted],
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
            TrackEntry {
                id: track.id(),
                start_s,
                duration_s,
                color: painted
                    .iter()
                    .find(|one| one.track == track.id())
                    .map(|one| [one.color.red, one.color.green, one.color.blue]),
                clip: outputs.and_then(|outputs| written(&outputs.clips)),
                zoom: outputs.and_then(|outputs| written(&outputs.zooms)),
            }
        })
        .collect()
}

fn relative(dest: &Path, path: &Path) -> Option<String> {
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
