use std::error::Error;
use std::path::Path;

use bench::guard::{Change, ClassGuard, Guard};
use bench::metrics::Evaluation;
use bench::stats::ClassSummary;
use bench::synthetic::{self, Class};
use engine::settings;
use tracking::error::Error as TrackingError;
use tracking::setting::Setting as TrackingSetting;

#[cfg(test)]
mod tests;

pub(crate) const ABOUT: &str = "Measures the detection on synthetic bats injected in a video";
pub(crate) const VIDEO_HELP: &str = "video the bats are injected in";
pub(crate) const OUT_DIR_HELP: &str = "folder that receives one folder per video, or the report of a replay";
pub(crate) const PER_CLASS_HELP: &str = "targets drawn per class";
pub(crate) const AMPLITUDES_HELP: &str = "contrast of the targets, grey levels; give the video before them";
pub(crate) const SIGMAS_HELP: &str = "blob size, source pixels";
pub(crate) const MOTIONS_HELP: &str = "pass: smooth crossing; hunt, circle: turns";
pub(crate) const SEED_HELP: &str = "seed of the drawing; the same seed draws the same targets on one system";
pub(crate) const MATCH_RADIUS_HELP: &str =
    "farthest a detection may lie from a target and still find it; source pixels";
pub(crate) const TARGETS_HELP: &str = "replays the targets of this file (bats.jsonl.gz) instead of drawing them";
pub(crate) const REPLAY_HELP: &str =
    "evaluates again a run written by an earlier bench, with other tracking settings, without any detection";
pub(crate) const AGAINST_HELP: &str = "replays the targets of a bench recorded by the v0.1.1 (bench.json, bats.jsonl.gz) \
     and lists the targets lost and gained; fails below the lower Wilson bound of a class";
pub(crate) const THRESHOLD_HELP: &str = "smallest difference from the background that counts as a target; lower finds \
     fainter targets and more noise; grey levels";
pub(crate) const MIN_AREA_HELP: &str = "smaller spots are dropped as noise; source pixels²";
pub(crate) const MAX_AREA_HELP: &str = "larger spots are dropped; source pixels²";
pub(crate) const BG_WINDOW_HELP: &str = "the background is the median of this much video around each frame; a target \
     that stays still for more than half of it blends into it; seconds";
pub(crate) const BG_STEP_HELP: &str = "the background uses one frame in this many; higher is faster but leaves fewer \
     frames to measure the noise of each pixel; frames";
pub(crate) const OSD_REGION_HELP: &str =
    "hides a moving on-screen display from the detection, as fractions of the frame; repeat for several";
pub(crate) const MERGE_RADIUS_HELP: &str = "spots up to about twice this distance apart become one detection, so a \
     fragmented target counts once; source pixels";
pub(crate) const WORK_WIDTH_HELP: &str =
    "width the frames are reduced to for detection; wider sees smaller targets but takes longer; pixels";
pub(crate) const NOISE_FACTOR_HELP: &str = "raises --threshold to this many times the noise of each pixel, so noisy \
     areas give fewer false tracks; 0 disables";
pub(crate) const TARGET_SIGMA_HELP: &str = "blur matched to the size of a small target before the comparison with \
     the background, so faint targets stand out from the noise; source pixels; 0 disables";
pub(crate) const MAX_BLOBS_HELP: &str = "frames with at least this many spots more than six times the usual count are \
     ignored, as when the binoculars move; 0 disables";
pub(crate) const UNSTABLE_PAD_HELP: &str = "time also ignored on each side of such frames; seconds";
pub(crate) const MAX_JUMP_HELP: &str =
    "farthest a detection may lie from where a track was expected and still extend it; source pixels";
pub(crate) const MAX_GAP_HELP: &str = "frames in a row without a detection that a track survives, so a faint target \
     that briefly disappears keeps one track; frames";
pub(crate) const MIN_HITS_HELP: &str =
    "tracks with fewer detections are dropped; lower keeps shorter tracks and more short flickers";
pub(crate) const MIN_TRAVEL_HELP: &str =
    "tracks that move less from start to end are dropped, like a pixel flickering in place; source pixels";
pub(crate) const TWIN_DISTANCE_HELP: &str = "two tracks that stay this close on all their shared frames are merged \
     as one animal; source pixels; 0 disables";
pub(crate) const MAX_MEDIAN_TURN_HELP: &str =
    "tracks that zigzag more are dropped as noise; radians; 3.15 or more disables";
pub(crate) const REJOIN_HIDDEN_HELP: &str = "a track that hides longer than --max-gap, up to this many frames, is \
     joined to the one that starts where both point; frames; 0 disables";
pub(crate) const REJOIN_DISTANCE_HELP: &str = "farthest the start of the second track may lie from where both tracks \
     point; source pixels; 0 disables";
pub(crate) const REJOIN_PIECE_POINTS_HELP: &str = "a track too short to keep, from this many points, may still \
     bridge or extend kept tracks when rejoining, never alone; points; 0 disables";
pub(crate) const REJOIN_JUNCTION_PACE_HELP: &str = "slowest jump over a hiding, as a share of the speed of each kept \
     track of the link; 0 accepts any";
pub(crate) const REJOIN_PIECE_PACE_HELP: &str = "slowest own speed of a piece of two points or more, as a share of the \
     speed of the kept track it continues; 0 accepts any";
pub(crate) const BENCH_HEADING: &str = "bench";
pub(crate) const DETECTION_HEADING: &str = "detection";
pub(crate) const TRACKING_HEADING: &str = "tracking";
pub(crate) const NO_VIDEO: &str = "a video is needed, unless --replay is given";
const GUARD_CONTRAST: &str = "visibility decided by the contrast the v0.1.1 recorded; \
     the table above and bench.json use the contrast the Rust measured";
const CAUSE_SEPARATOR: &str = ": ";
const TABLE_HEADER: &str =
    "motion  amplitude  sigma   n  found  ci95            completeness  start_delay  end_early  fragments  effective";
const MISSING: &str = "-";

pub(crate) fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub(crate) fn causes(error: &dyn Error) -> String {
    let mut text = error.to_string();
    let mut source = error.source();
    while let Some(cause) = source {
        text.push_str(CAUSE_SEPARATOR);
        text.push_str(&cause.to_string());
        source = cause.source();
    }
    text
}

pub(crate) fn failed(video: &Path, error: &dyn Error) -> String {
    format!("{}{CAUSE_SEPARATOR}{}", video.display(), causes(error))
}

fn tracking_option(setting: TrackingSetting) -> &'static str {
    match setting {
        TrackingSetting::PadS => "--unstable-pad",
        TrackingSetting::MaxJump => "--max-jump",
        TrackingSetting::MinTravel => "--min-travel",
        TrackingSetting::TwinDistance => "--twin-distance",
        TrackingSetting::MaxMedianTurn => "--max-median-turn",
        TrackingSetting::RejoinHidden => "--rejoin-hidden",
        TrackingSetting::RejoinDistance => "--rejoin-distance",
        TrackingSetting::RejoinJunctionPace => "--rejoin-junction-pace",
        TrackingSetting::RejoinPiecePace => "--rejoin-piece-pace",
    }
}

pub(crate) fn detection_refused(error: &settings::Error) -> String {
    match *error {
        settings::Error::Setting { setting, value, bound } => {
            format!("--{} must be {bound}, not {value}", setting.flag())
        }
        settings::Error::Areas { min_area, max_area } => {
            format!("--min-area {min_area} is larger than --max-area {max_area}")
        }
        settings::Error::Region { index, region } => format!(
            "--osd-region {} does not satisfy 0 ≤ x0 < x1 ≤ 1 and 0 ≤ y0 < y1 ≤ 1: {region}",
            index + 1
        ),
    }
}

pub(crate) fn tracking_refused(error: &TrackingError) -> String {
    match error {
        TrackingError::Setting { setting, value } => {
            format!("{} is out of bounds: {value}", tracking_option(*setting))
        }
        TrackingError::Track(_) => error.to_string(),
    }
}

pub(crate) fn match_refused(error: &bench::config::Error) -> String {
    match error {
        bench::config::Error::Radius(radius) => format!("--match-radius must be a finite number > 0, not {radius}"),
        bench::config::Error::Purity(_) => error.to_string(),
    }
}

pub(crate) fn class_refused(error: &synthetic::Error) -> String {
    match error {
        synthetic::Error::Amplitude(amplitude) => format!("--amplitudes must be finite numbers, not {amplitude}"),
        synthetic::Error::Sigma(sigma) => format!("--sigmas must be finite numbers > 0, not {sigma}"),
        synthetic::Error::Lengths { .. } | synthetic::Error::Empty { .. } | synthetic::Error::TooLong { .. } => {
            error.to_string()
        }
    }
}

fn optional(value: Option<f64>, precision: usize) -> String {
    value.map_or_else(|| MISSING.to_owned(), |value| format!("{value:.precision$}"))
}

pub(crate) fn table(rows: &[ClassSummary], evaluation: &Evaluation) -> String {
    let mut lines = vec![TABLE_HEADER.to_owned()];
    for row in rows {
        let class = row.class;
        let ci95 = format!("[{:.3}, {:.3}]", row.found_ci95.0, row.found_ci95.1);
        lines.push(format!(
            "{:<6}  {:>9.1}  {:>5.1}  {:>2}  {:>5}  {ci95:<14}  {:>12.3}  {:>11}  {:>9}  {:>9}  {:>9.1}",
            class.motion().name(),
            class.amplitude(),
            class.sigma(),
            row.n,
            row.found,
            row.median_completeness,
            optional(row.median_start_delay, 1),
            optional(row.median_end_early, 1),
            optional(row.mean_fragments, 2),
            row.median_effective,
        ));
    }
    lines.push(format!(
        "not visible: {}  false tracks: {}  tracks: {} injected run, {} reference",
        evaluation.not_visible, evaluation.false_tracks, evaluation.injected_tracks, evaluation.reference_tracks
    ));
    lines.join("\n")
}

fn class_name(class: Class) -> String {
    format!("{} {} σ {}", class.motion().name(), class.amplitude(), class.sigma())
}

fn changes(changes: &[Change]) -> String {
    if changes.is_empty() {
        return MISSING.to_owned();
    }
    changes
        .iter()
        .map(|change| format!("{} (frame {})", change.id, change.start.get()))
        .collect::<Vec<_>>()
        .join(", ")
}

fn class_guard(guarded: &ClassGuard) -> String {
    let verdict = if guarded.failed() { "below the bound" } else { "passes" };
    format!(
        "{}: {} visible on both sides, v0.1.1 found {}, Rust found {}, lower bound {:.3}: {verdict}\n  lost: {}\n  gained: {}",
        class_name(guarded.class),
        guarded.n,
        guarded.v011_found,
        guarded.found,
        guarded.lowest,
        changes(&guarded.lost),
        changes(&guarded.gained),
    )
}

pub(crate) fn guard(folder: &Path, guard: &Guard) -> String {
    let verdict = if guard.failed() { "FAILED" } else { "passed" };
    let mut lines = vec![
        format!("guard against {}: {verdict}", folder.display()),
        GUARD_CONTRAST.to_owned(),
    ];
    lines.extend(guard.classes.iter().map(class_guard));
    lines.push(format!(
        "visible for the v0.1.1 only ({}): {}\nvisible for the Rust only ({}): {}\n\
         visibility changed by the contrast the Rust measured ({}): {}\nfalse tracks: v0.1.1 {}, Rust {}\n\
         reference tracks: v0.1.1 {}, Rust {}",
        guard.v011_only.len(),
        changes(&guard.v011_only),
        guard.rust_only.len(),
        changes(&guard.rust_only),
        guard.contrast_differs.len(),
        changes(&guard.contrast_differs),
        guard.v011_false_tracks,
        guard.false_tracks,
        guard.v011_reference_tracks,
        guard.reference_tracks,
    ));
    lines.join("\n")
}
