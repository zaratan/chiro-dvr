use engine::settings::Region;
use tracking::error::Error;
use tracking::exclusion::exclude;
use tracking::stability::StabilityConfig;
use tracking::track::{TrackConfig, track_detections};
use types::frame::FrameIndex;
use types::span::Span;
use types::track::Track;

use crate::config::MatchConfig;
use crate::matching::{assign_tracks, matches_reference, near, reference_points, truth};
use crate::median::median;
use crate::run::Run;
use crate::synthetic::injection::{Injected, Observation};
use crate::synthetic::{Class, Scene};

#[cfg(test)]
mod tests;

pub(crate) const VISIBLE_MIN_CONTRAST: f64 = 8.0;
pub(crate) const VISIBLE_MARGIN_SIGMAS: f64 = 2.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TargetResult {
    pub id: usize,
    pub class: Class,
    pub visible_frames: usize,
    pub found: bool,
    pub completeness: f64,
    pub start_delay: Option<usize>,
    pub end_early: Option<usize>,
    pub fragments: usize,
    pub median_effective: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Evaluation {
    pub targets: Vec<TargetResult>,
    pub not_visible: usize,
    pub false_tracks: usize,
    pub reference_tracks: usize,
    pub injected_tracks: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Judging {
    pub track: TrackConfig,
    pub stability: StabilityConfig,
    pub matching: MatchConfig,
}

pub(crate) fn is_visible(observation: &Observation, sigma: f64, scene: Scene, masked: &[Region]) -> bool {
    let margin = VISIBLE_MARGIN_SIGMAS * sigma;
    let Observation { at, effective, .. } = *observation;
    let inside =
        (margin..=scene.width() - margin).contains(&at.x) && (margin..=scene.height() - margin).contains(&at.y);
    inside
        && !masked
            .iter()
            .any(|region| region.contains(at.x / scene.width(), at.y / scene.height()))
        && effective.abs() >= VISIBLE_MIN_CONTRAST
}

pub fn evaluate(run: &Run, settings: Judging) -> Result<Evaluation, Error> {
    evaluate_with(run, &run.targets, settings)
}

pub fn evaluate_with(run: &Run, observed: &[Injected], settings: Judging) -> Result<Evaluation, Error> {
    let video = &run.video;
    let excluded = |detections| {
        exclude(
            detections,
            video.damaged.clone(),
            video.reported_frames,
            video.rate,
            settings.stability,
        )
    };
    let injected = excluded(&run.injected);
    let ignored: Vec<Span> = injected.ignored_spans().collect();
    let injected_tracks = track_detections(&injected.detections, settings.track)?;
    let reference_tracks = track_detections(&excluded(&run.reference).detections, settings.track)?;
    let assigned = assign_tracks(&injected_tracks, &truth(observed), settings.matching);
    let mut targets = Vec::new();
    let mut not_visible = 0;
    for (index, placed) in observed.iter().enumerate() {
        let visible: Vec<&Observation> = placed
            .observations
            .iter()
            .filter(|observation| {
                is_visible(
                    observation,
                    placed.target.class().sigma(),
                    video.scene(),
                    &run.detect.osd_regions,
                ) && !ignored.iter().any(|span| span.contains(observation.frame))
            })
            .collect();
        let fragments: Vec<&Track> = injected_tracks
            .iter()
            .zip(&assigned)
            .filter(|&(_, target)| *target == Some(index))
            .map(|(track, _)| track)
            .collect();
        match result(placed, &visible, &fragments, settings) {
            Some(found) => targets.push(found),
            None => not_visible += 1,
        }
    }
    let points = reference_points(&reference_tracks);
    let false_tracks = injected_tracks
        .iter()
        .zip(&assigned)
        .filter(|&(track, target)| target.is_none() && !matches_reference(track, &points, settings.matching))
        .count();
    Ok(Evaluation {
        targets,
        not_visible,
        false_tracks,
        reference_tracks: reference_tracks.len(),
        injected_tracks: injected_tracks.len(),
    })
}

fn result(
    placed: &Injected,
    visible: &[&Observation],
    fragments: &[&Track],
    settings: Judging,
) -> Option<TargetResult> {
    let min_hits = settings.track.min_hits().get();
    if visible.len() < min_hits {
        return None;
    }
    let seen_at = |frame: FrameIndex| {
        visible
            .binary_search_by_key(&frame, |observation| observation.frame)
            .ok()
            .map(|index| visible[index].at)
    };
    let mut covered: Vec<FrameIndex> = fragments
        .iter()
        .flat_map(|track| track.points())
        .filter(|detection| seen_at(detection.frame).is_some_and(|at| near(detection, at, settings.matching.radius())))
        .map(|detection| detection.frame)
        .collect();
    covered.sort_unstable();
    covered.dedup();
    let found = covered.len() >= min_hits;
    let first_visible = visible[0].frame;
    let last_visible = visible[visible.len() - 1].frame;
    Some(TargetResult {
        id: placed.target.id(),
        class: placed.target.class(),
        visible_frames: visible.len(),
        found,
        completeness: covered.len() as f64 / visible.len() as f64,
        start_delay: found.then(|| covered[0].get() - first_visible.get()),
        end_early: found.then(|| last_visible.get() - covered[covered.len() - 1].get()),
        fragments: fragments.len(),
        median_effective: median(visible.iter().map(|observation| observation.effective)).unwrap_or(0.0),
    })
}
