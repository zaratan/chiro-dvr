use std::num::NonZeroU32;

use draw::canvas::{Canvas, PixelBox, Point};
use draw::color::Rgb;
use media::convert::Region;
use media::frame::YuvFrame;
use types::detection::Detection;
use types::frame::FrameIndex;
use types::track::Track;

use crate::clips::Error;
use crate::picture::Marks;
use crate::positions::{bounded, filled};
use crate::zones::{even_region, merged};

#[cfg(test)]
mod tests;

pub(crate) const SLOW_MOTION: NonZeroU32 = NonZeroU32::MIN.saturating_add(3);
const MARGIN: f64 = 40.0;
const MAX_MAGNIFICATION: u32 = 4;
const TRAIL_GAP: f64 = 16.0;
const TRAIL_WIDTH: f32 = 1.0;
const PIXEL_CENTER: f64 = 0.5;
const ZONE_MARGIN: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Crop {
    pub(crate) left: u32,
    pub(crate) top: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub(crate) fn crop(track: &Track, width: u32, height: u32) -> Crop {
    let points = bounded(track.points());
    let fold =
        |pick: fn(&Detection) -> f64, start: f64, keep: fn(f64, f64) -> f64| points.iter().map(pick).fold(start, keep);
    let x0 = fold(|point| point.left, f64::INFINITY, f64::min) - MARGIN;
    let y0 = fold(|point| point.top, f64::INFINITY, f64::min) - MARGIN;
    let x1 = fold(|point| point.left + point.width, f64::NEG_INFINITY, f64::max) + MARGIN;
    let y1 = fold(|point| point.top + point.height, f64::NEG_INFINITY, f64::max) + MARGIN;
    let units = gcd(width, height).max(1);
    let (unit_width, unit_height) = (width / units, height / units);
    let needed = ((x1 - x0) / f64::from(unit_width))
        .ceil()
        .max(((y1 - y0) / f64::from(unit_height)).ceil()) as u32;
    let count = needed.max(units.div_ceil(MAX_MAGNIFICATION)).min(units);
    let (crop_width, crop_height) = (count * unit_width, count * unit_height);
    Crop {
        left: placed(f64::midpoint(x0, x1), crop_width, width),
        top: placed(f64::midpoint(y0, y1), crop_height, height),
        width: crop_width,
        height: crop_height,
    }
}

fn placed(center: f64, size: u32, limit: u32) -> u32 {
    let start = (center - f64::from(size) / 2.0).round().max(0.0) as u32;
    start.min(limit.saturating_sub(size))
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[derive(Debug)]
pub(crate) struct Magnifier {
    width: u32,
    height: u32,
    luma_columns: Vec<usize>,
    luma_rows: Vec<usize>,
    chroma_columns: Vec<usize>,
    chroma_rows: Vec<usize>,
}

impl Magnifier {
    pub(crate) fn new(crop: Crop, width: u32, height: u32) -> Self {
        let luma_columns = picks(crop.left, crop.width, width);
        let luma_rows = picks(crop.top, crop.height, height);
        Self {
            width,
            height,
            chroma_columns: halved(&luma_columns),
            chroma_rows: halved(&luma_rows),
            luma_columns,
            luma_rows,
        }
    }

    pub(crate) fn magnify<'a>(&self, source: &YuvFrame, into: &'a mut Option<YuvFrame>) -> Result<&'a YuvFrame, Error> {
        if (source.width(), source.height()) != (self.width, self.height) {
            return Err(Error::ZoomSize {
                width: self.width,
                height: self.height,
                found_width: source.width(),
                found_height: source.height(),
            });
        }
        let zoomed = match into {
            Some(zoomed) => zoomed,
            None => into.insert(YuvFrame::new(self.width, self.height, source.range()).map_err(Error::Frame)?),
        };
        let (width, chroma_width) = (self.width as usize, self.width.div_ceil(2) as usize);
        sample(source.y(), width, zoomed.y_mut(), &self.luma_columns, &self.luma_rows);
        sample(
            source.u(),
            chroma_width,
            zoomed.u_mut(),
            &self.chroma_columns,
            &self.chroma_rows,
        );
        sample(
            source.v(),
            chroma_width,
            zoomed.v_mut(),
            &self.chroma_columns,
            &self.chroma_rows,
        );
        Ok(zoomed)
    }
}

fn picks(start: u32, size: u32, out: u32) -> Vec<usize> {
    (0..u64::from(out))
        .map(|index| u64::from(start) + index * u64::from(size) / u64::from(out))
        .map(|pick| pick as usize)
        .collect()
}

fn halved(luma: &[usize]) -> Vec<usize> {
    luma.iter().step_by(2).map(|pick| pick / 2).collect()
}

fn sample(source: &[u8], source_row: usize, out: &mut [u8], columns: &[usize], rows: &[usize]) {
    let width = columns.len();
    let mut last = None;
    for (index, row) in rows.iter().enumerate() {
        let start = index * width;
        if last == Some(*row) && out.len() >= start + width {
            out.copy_within(start - width..start, start);
            continue;
        }
        let from = source.chunks_exact(source_row).nth(*row).unwrap_or_default();
        let line = out.get_mut(start..start + width).unwrap_or_default();
        for (value, column) in line.iter_mut().zip(columns) {
            *value = from.get(*column).copied().unwrap_or_default();
        }
        last = Some(*row);
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Spot {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Piece {
    from: Spot,
    to: Spot,
}

#[derive(Debug)]
pub(crate) struct ZoomTrail {
    crop: Crop,
    scale_x: f64,
    scale_y: f64,
    first: FrameIndex,
    positions: Vec<Detection>,
    detected: Vec<Detection>,
    trail_frames: usize,
    color: Rgb,
}

impl ZoomTrail {
    pub(crate) fn new(track: &Track, color: Rgb, crop: Crop, size: (u32, u32), trail_frames: usize) -> Self {
        let positions: Vec<Detection> = filled(&bounded(track.points()))
            .into_iter()
            .map(|(position, _)| position)
            .collect();
        Self {
            crop,
            scale_x: f64::from(size.0) / f64::from(crop.width),
            scale_y: f64::from(size.1) / f64::from(crop.height),
            first: track.first().frame,
            positions,
            detected: track.points().to_vec(),
            trail_frames,
            color,
        }
    }

    pub(crate) fn pieces(&self, frame: FrameIndex) -> Vec<Piece> {
        let Some(current) = frame
            .get()
            .checked_sub(self.first.get())
            .and_then(|offset| self.positions.get(offset))
        else {
            return Vec::new();
        };
        let since = FrameIndex::new(frame.get().saturating_sub(self.trail_frames));
        let from = self.detected.partition_point(|point| point.frame < since);
        let to = self.detected.partition_point(|point| point.frame < frame);
        let path: Vec<Spot> = self
            .detected
            .get(from..to)
            .unwrap_or_default()
            .iter()
            .chain(std::iter::once(current))
            .map(|point| Spot { x: point.x, y: point.y })
            .collect();
        let low = Spot {
            x: current.left - TRAIL_GAP,
            y: current.top - TRAIL_GAP,
        };
        let high = Spot {
            x: current.left + current.width + TRAIL_GAP,
            y: current.top + current.height + TRAIL_GAP,
        };
        path.iter()
            .zip(path.iter().skip(1))
            .filter(|(from, to)| from != to)
            .flat_map(|(from, to)| outside(Piece { from: *from, to: *to }, low, high))
            .flatten()
            .map(|piece| Piece {
                from: self.on_screen(piece.from),
                to: self.on_screen(piece.to),
            })
            .collect()
    }

    fn on_screen(&self, spot: Spot) -> Spot {
        Spot {
            x: (spot.x - f64::from(self.crop.left) + PIXEL_CENTER) * self.scale_x,
            y: (spot.y - f64::from(self.crop.top) + PIXEL_CENTER) * self.scale_y,
        }
    }
}

impl Marks for ZoomTrail {
    fn zones(&self, frame: FrameIndex, width: u32, height: u32) -> Vec<Region> {
        let reach = TRAIL_WIDTH.ceil() as i32 + ZONE_MARGIN;
        let zones = self
            .pieces(frame)
            .iter()
            .filter_map(|piece| {
                let extent = PixelBox {
                    left: (piece.from.x.min(piece.to.x).floor() as i32).saturating_sub(reach),
                    top: (piece.from.y.min(piece.to.y).floor() as i32).saturating_sub(reach),
                    right: (piece.from.x.max(piece.to.x).ceil() as i32).saturating_add(reach),
                    bottom: (piece.from.y.max(piece.to.y).ceil() as i32).saturating_add(reach),
                };
                even_region(extent, width, height)
            })
            .collect();
        merged(zones, width, height)
    }

    fn draw(&self, canvas: &mut Canvas, frame: FrameIndex) {
        for piece in self.pieces(frame) {
            let point = |spot: Spot| Point {
                x: spot.x as f32,
                y: spot.y as f32,
            };
            canvas.line(point(piece.from), point(piece.to), self.color, TRAIL_WIDTH);
        }
    }
}

fn outside(piece: Piece, low: Spot, high: Spot) -> [Option<Piece>; 2] {
    let Some((enter, leave)) = inside_span(piece, low, high) else {
        return [Some(piece), None];
    };
    let at = |share: f64| Spot {
        x: piece.from.x + (piece.to.x - piece.from.x) * share,
        y: piece.from.y + (piece.to.y - piece.from.y) * share,
    };
    [
        (enter > 0.0).then(|| Piece {
            from: piece.from,
            to: at(enter),
        }),
        (leave < 1.0).then(|| Piece {
            from: at(leave),
            to: piece.to,
        }),
    ]
}

fn inside_span(piece: Piece, low: Spot, high: Spot) -> Option<(f64, f64)> {
    let (mut enter, mut leave) = (0.0_f64, 1.0_f64);
    let axes = [
        (piece.from.x, piece.to.x, low.x, high.x),
        (piece.from.y, piece.to.y, low.y, high.y),
    ];
    for (from, to, low, high) in axes {
        let delta = to - from;
        if delta == 0.0 {
            if !(low..=high).contains(&from) {
                return None;
            }
            continue;
        }
        let (first, second) = ((low - from) / delta, (high - from) / delta);
        enter = enter.max(first.min(second));
        leave = leave.min(first.max(second));
    }
    (enter < leave).then_some((enter, leave))
}
