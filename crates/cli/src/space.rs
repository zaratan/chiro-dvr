use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

pub const OUTPUTS_PER_VIDEO_BYTE: f64 = 1.0;
pub const ANNOTATED_PER_VIDEO_BYTE: f64 = 2.8;
const CLIP_PER_SOURCE_BYTE: f64 = 3.2;
const ZOOM_PER_SOURCE_BYTE: f64 = 2.7;
pub const GIGABYTE: u64 = 1_000_000_000;
const MARGIN_DIVISOR: u64 = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Need {
    pub folder: PathBuf,
    pub bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Volume {
    pub device: u64,
    pub free: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Short {
    pub folder: PathBuf,
    pub needed: u64,
    pub free: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rendering {
    pub clip_s: f64,
    pub zoom_s: f64,
    pub annotated: bool,
}

pub fn launch_estimate(video_bytes: u64, annotated: bool) -> u64 {
    let share = if annotated {
        OUTPUTS_PER_VIDEO_BYTE + ANNOTATED_PER_VIDEO_BYTE
    } else {
        OUTPUTS_PER_VIDEO_BYTE
    };
    bytes(video_bytes as f64 * share)
}

pub fn with_margin(estimate: u64) -> u64 {
    estimate.saturating_add(estimate.div_ceil(MARGIN_DIVISOR).max(GIGABYTE))
}

pub fn render_estimate(video_bytes: u64, length_s: f64, rendering: Rendering) -> u64 {
    let video = video_bytes as f64;
    let annotated = if rendering.annotated {
        video * ANNOTATED_PER_VIDEO_BYTE
    } else {
        0.0
    };
    if length_s.is_nan() || length_s <= 0.0 {
        return bytes(annotated);
    }
    let per_second = video / length_s;
    bytes(per_second * (rendering.clip_s * CLIP_PER_SOURCE_BYTE + rendering.zoom_s * ZOOM_PER_SOURCE_BYTE) + annotated)
}

pub fn short_volumes(needs: &[Need], volume_of: impl Fn(&Path) -> Option<Volume>) -> Vec<Short> {
    let mut volumes: Vec<(Volume, &Path, u64)> = Vec::new();
    for need in needs {
        let Some(volume) = volume_of(&need.folder) else {
            continue;
        };
        match volumes.iter_mut().find(|(known, _, _)| known.device == volume.device) {
            Some((_, _, total)) => *total = total.saturating_add(need.bytes),
            None => volumes.push((volume, &need.folder, need.bytes)),
        }
    }
    volumes
        .into_iter()
        .filter_map(|(volume, folder, total)| {
            let needed = with_margin(total);
            (needed > volume.free).then(|| Short {
                folder: folder.to_path_buf(),
                needed,
                free: volume.free,
            })
        })
        .collect()
}

fn bytes(value: f64) -> u64 {
    value.ceil() as u64
}
