use draw::canvas::{Canvas, PixelBox, Point};
use draw::color::{BOX, TRAIL};
use draw::style::TRACK_LABEL;
use draw::text::{ShapedText, Typeface};
use media::convert::Region;
use types::detection::Detection;
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::track::Track;

use crate::config::RenderConfig;
use crate::picture::Marks;
use crate::positions::{bounded, filled};
use crate::zones::{even_region, merged};

#[cfg(test)]
mod tests;

const LABEL_LIFT: i32 = 6;
const BOX_WIDTH: u32 = 2;
const INTERPOLATED_BOX_WIDTH: u32 = 1;
const TRAIL_WIDTH: f32 = 2.0;
const PIXEL_CENTER: f64 = 0.5;
const LABEL_HEIGHTS: f32 = 2.0;
const ZONE_MARGIN: i32 = 4;

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

impl Marks for Overlay {
    fn draw(&self, canvas: &mut Canvas, frame: FrameIndex) {
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

    fn zones(&self, frame: FrameIndex, width: u32, height: u32) -> Vec<Region> {
        let Some(marks) = self.frames.get(frame.get()) else {
            return Vec::new();
        };
        let zones = marks
            .iter()
            .filter_map(|mark| {
                let track = self.tracks.get(mark.track)?;
                even_region(self.extent(track, mark), width, height)
            })
            .collect();
        merged(zones, width, height)
    }
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

    fn extent(&self, track: &Marked, mark: &Mark) -> PixelBox {
        let area = self.area(&mark.detection);
        let grow = i32::try_from(BOX_WIDTH).unwrap_or(i32::MAX);
        let baseline = area.top.saturating_sub(LABEL_LIFT);
        let label_height = (TRACK_LABEL.digit_px() * LABEL_HEIGHTS).ceil() as i32;
        let label_right = area.left.saturating_add(track.label.width().ceil() as i32);
        let mut extent = PixelBox {
            left: area.left.saturating_sub(grow),
            top: area.top.saturating_sub(grow).min(baseline.saturating_sub(label_height)),
            right: area.right.saturating_add(grow).max(label_right),
            bottom: area.bottom.saturating_add(grow),
        };
        let reach = TRAIL_WIDTH.ceil() as i32;
        for point in self.trail(track, mark) {
            extent.left = extent.left.min((point.x.floor() as i32).saturating_sub(reach));
            extent.top = extent.top.min((point.y.floor() as i32).saturating_sub(reach));
            extent.right = extent.right.max((point.x.ceil() as i32).saturating_add(reach));
            extent.bottom = extent.bottom.max((point.y.ceil() as i32).saturating_add(reach));
        }
        PixelBox {
            left: extent.left.saturating_sub(ZONE_MARGIN),
            top: extent.top.saturating_sub(ZONE_MARGIN),
            right: extent.right.saturating_add(ZONE_MARGIN),
            bottom: extent.bottom.saturating_add(ZONE_MARGIN),
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
