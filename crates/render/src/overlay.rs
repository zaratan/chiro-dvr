use draw::canvas::{Canvas, PixelBox, Point};
use draw::color::{BOX, TRAIL};
use draw::style::TRACK_LABEL;
use draw::text::{ShapedText, Typeface};
use types::detection::Detection;
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::track::Track;

use crate::config::RenderConfig;

#[cfg(test)]
mod tests;

const LABEL_LIFT: i32 = 6;
const BOX_WIDTH: u32 = 2;
const INTERPOLATED_BOX_WIDTH: u32 = 1;
const TRAIL_WIDTH: f32 = 2.0;
const PIXEL_CENTER: f64 = 0.5;
const BOX_SPREAD_LIMIT: f64 = 4.0;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Mark {
    track: usize,
    detection: Detection,
    interpolated: bool,
}

#[derive(Debug)]
struct Marked {
    label: ShapedText,
    centers: Vec<(FrameIndex, Point)>,
}

#[derive(Debug)]
pub(crate) struct Overlay {
    tracks: Vec<Marked>,
    frames: Vec<Vec<Mark>>,
    box_pad: i32,
    trail_frames: usize,
}

impl Overlay {
    pub(crate) fn new(
        tracks: &[Track],
        frame_count: usize,
        config: RenderConfig,
        rate: FrameRate,
        typeface: &Typeface,
    ) -> Self {
        let mut frames = vec![Vec::new(); frame_count];
        for (index, track) in tracks.iter().enumerate() {
            for (detection, interpolated) in filled(&bounded(track.points())) {
                if let Some(marks) = frames.get_mut(detection.frame.get()) {
                    marks.push(Mark {
                        track: index,
                        detection,
                        interpolated,
                    });
                }
            }
        }
        let tracks = tracks
            .iter()
            .map(|track| Marked {
                label: typeface.shape(&format!("#{}", track.id()), TRACK_LABEL),
                centers: track
                    .points()
                    .iter()
                    .map(|point| (point.frame, center(point)))
                    .collect(),
            })
            .collect();
        Self {
            tracks,
            frames,
            box_pad: i32::try_from(config.box_pad()).unwrap_or(i32::MAX),
            trail_frames: config.trail_frames(rate),
        }
    }

    pub(crate) fn draw(&self, canvas: &mut Canvas, frame: FrameIndex) {
        let Some(marks) = self.frames.get(frame.get()) else {
            return;
        };
        for mark in marks {
            let Some(track) = self.tracks.get(mark.track) else {
                continue;
            };
            let area = self.area(&mark.detection);
            let width = if mark.interpolated {
                INTERPOLATED_BOX_WIDTH
            } else {
                BOX_WIDTH
            };
            canvas.rectangle(area, BOX, width);
            let baseline = Point {
                x: area.left as f32,
                y: area.top.saturating_sub(LABEL_LIFT) as f32,
            };
            canvas.shaped_text(&track.label, baseline, BOX);
            let trail = self.trail(track, mark);
            if trail.len() > 1 {
                canvas.polyline(&trail, TRAIL, TRAIL_WIDTH);
            }
        }
    }

    fn area(&self, detection: &Detection) -> PixelBox {
        PixelBox {
            left: (detection.left as i32).saturating_sub(self.box_pad),
            top: (detection.top as i32).saturating_sub(self.box_pad),
            right: ((detection.left + detection.width) as i32).saturating_add(self.box_pad),
            bottom: ((detection.top + detection.height) as i32).saturating_add(self.box_pad),
        }
    }

    fn trail(&self, track: &Marked, mark: &Mark) -> Vec<Point> {
        let now = mark.detection.frame;
        let since = FrameIndex::new(now.get().saturating_sub(self.trail_frames));
        let from = track.centers.partition_point(|(frame, _)| *frame < since);
        let to = track.centers.partition_point(|(frame, _)| *frame < now);
        let past = track.centers.get(from..to).unwrap_or_default();
        past.iter()
            .map(|(_, point)| *point)
            .chain(std::iter::once(center(&mark.detection)))
            .collect()
    }
}

fn center(detection: &Detection) -> Point {
    Point {
        x: (detection.x + PIXEL_CENTER) as f32,
        y: (detection.y + PIXEL_CENTER) as f32,
    }
}

fn bounded(points: &[Detection]) -> Vec<Detection> {
    let widest = median(points.iter().map(|point| point.width)).map(|width| BOX_SPREAD_LIMIT * width);
    let tallest = median(points.iter().map(|point| point.height)).map(|height| BOX_SPREAD_LIMIT * height);
    points
        .iter()
        .map(|point| {
            let mut point = *point;
            if let Some(widest) = widest.filter(|widest| point.width > *widest) {
                (point.left, point.width) = (point.x - widest / 2.0, widest);
            }
            if let Some(tallest) = tallest.filter(|tallest| point.height > *tallest) {
                (point.top, point.height) = (point.y - tallest / 2.0, tallest);
            }
            point
        })
        .collect()
}

fn median(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut values: Vec<f64> = values.collect();
    values.sort_by(f64::total_cmp);
    let upper = values.get(values.len() / 2).copied()?;
    if values.len() % 2 == 1 {
        return Some(upper);
    }
    let lower = values.get(values.len() / 2 - 1).copied()?;
    Some(f64::midpoint(lower, upper))
}

fn filled(points: &[Detection]) -> Vec<(Detection, bool)> {
    let mut filled = Vec::with_capacity(points.len());
    for (before, after) in points.iter().zip(points.iter().skip(1)) {
        filled.push((*before, false));
        let steps = after.frame.get().saturating_sub(before.frame.get());
        filled.extend((1..steps).map(|step| (between(before, after, step, steps), true)));
    }
    if let Some(last) = points.last() {
        filled.push((*last, false));
    }
    filled
}

fn between(before: &Detection, after: &Detection, step: usize, steps: usize) -> Detection {
    let share = step as f64 / steps as f64;
    let lerp = |from: f64, to: f64| from + (to - from) * share;
    Detection {
        frame: FrameIndex::new(before.frame.get() + step),
        x: lerp(before.x, after.x),
        y: lerp(before.y, after.y),
        left: lerp(before.left, after.left),
        top: lerp(before.top, after.top),
        width: lerp(before.width, after.width),
        height: lerp(before.height, after.height),
        area: lerp(before.area, after.area),
        amplitude: 0.0,
    }
}
