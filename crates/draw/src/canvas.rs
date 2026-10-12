use tiny_skia::{
    BlendMode, FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, PixmapPaint, Rect, Stroke, Transform,
};

use crate::color::Rgb;
use crate::text::{ShapedText, TextStyle, Typeface};

#[cfg(test)]
mod tests;

const BACKGROUND_DIM: f32 = 0.6;
const CHANNELS: usize = 4;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("a canvas cannot be {width}×{height}")]
    Size { width: u32, height: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Segment {
    pub from: Point,
    pub to: Point,
    pub width: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PixelBox {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Outline {
    pub color: Rgb,
    pub width: f32,
}

/// An opaque RGBA image: every alpha stays 255, so its bytes are plain RGBA as well as premultiplied.
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    pixmap: Pixmap,
}

impl Canvas {
    pub fn new(width: u32, height: u32, background: Rgb) -> Result<Self, Error> {
        let mut pixmap = Pixmap::new(width, height).ok_or(Error::Size { width, height })?;
        pixmap.fill(tiny_skia::Color::from_rgba8(
            background.red,
            background.green,
            background.blue,
            u8::MAX,
        ));
        Ok(Self { pixmap })
    }

    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }

    pub fn rgba(&self) -> &[u8] {
        self.pixmap.data()
    }

    /// Whoever writes these bytes keeps every alpha at 255.
    pub fn rgba_mut(&mut self) -> &mut [u8] {
        self.pixmap.data_mut()
    }

    /// Covers the box's own border pixels; each extra pixel of width grows the border outward.
    pub fn rectangle(&mut self, area: PixelBox, color: Rgb, width: u32) {
        if width == 0 || area.right < area.left || area.bottom < area.top {
            return;
        }
        let grow = i32::try_from(width - 1).unwrap_or(i32::MAX);
        let outer = PixelBox {
            left: area.left.saturating_sub(grow),
            top: area.top.saturating_sub(grow),
            right: area.right.saturating_add(grow),
            bottom: area.bottom.saturating_add(grow),
        };
        let bands = [
            PixelBox {
                bottom: area.top,
                ..outer
            },
            PixelBox {
                top: area.bottom,
                ..outer
            },
            PixelBox {
                right: area.left,
                ..outer
            },
            PixelBox {
                left: area.right,
                ..outer
            },
        ];
        for band in bands {
            self.fill_rectangle(band, color);
        }
    }

    pub fn fill_rectangle(&mut self, area: PixelBox, color: Rgb) {
        if area.right < area.left || area.bottom < area.top {
            return;
        }
        let pixels = Rect::from_ltrb(
            coordinate(area.left),
            coordinate(area.top),
            coordinate(area.right) + 1.0,
            coordinate(area.bottom) + 1.0,
        );
        let mut paint = paint(color);
        paint.anti_alias = false;
        if let Some(rect) = pixels {
            self.pixmap.fill_rect(rect, &paint, Transform::identity(), None);
        }
    }

    pub fn paste(&mut self, source: &Canvas, left: i32, top: i32) {
        if left > last_index(self.width()) || top > last_index(self.height()) {
            return;
        }
        let copy = PixmapPaint {
            blend_mode: BlendMode::Source,
            ..PixmapPaint::default()
        };
        self.pixmap
            .draw_pixmap(left, top, source.pixmap.as_ref(), &copy, Transform::identity(), None);
    }

    pub fn line(&mut self, from: Point, to: Point, color: Rgb, width: f32) {
        self.polyline(&[from, to], color, width);
    }

    pub fn polyline(&mut self, points: &[Point], color: Rgb, width: f32) {
        if let Some(path) = traced(points).and_then(PathBuilder::finish) {
            self.stroke(&path, color, width);
        }
    }

    pub fn segments(&mut self, segments: &[Segment], color: Rgb) {
        for segment in segments {
            self.line(segment.from, segment.to, color, segment.width);
        }
    }

    pub fn disk(&mut self, center: Point, radius: f32, fill: Rgb, outline: Option<Outline>) {
        if let Some(path) = PathBuilder::from_circle(center.x, center.y, radius) {
            self.shape(&path, Some(fill), outline);
        }
    }

    pub fn polygon(&mut self, points: &[Point], fill: Option<Rgb>, outline: Option<Outline>) {
        let closed = traced(points).and_then(|mut builder| {
            builder.close();
            builder.finish()
        });
        if let Some(path) = closed {
            self.shape(&path, fill, outline);
        }
    }

    pub fn text(&mut self, typeface: &Typeface, text: &str, baseline_start: Point, style: TextStyle, color: Rgb) {
        self.shaped_text(&typeface.shape(text, style), baseline_start, color);
    }

    pub fn shaped_text(&mut self, text: &ShapedText, baseline_start: Point, color: Rgb) {
        let paint = paint(color);
        for (path, left) in &text.glyphs {
            let at = Transform::from_translate(baseline_start.x + left, baseline_start.y);
            self.pixmap.fill_path(path, &paint, FillRule::Winding, at, None);
        }
    }

    pub fn dim(&mut self) {
        let (pixels, _) = self.pixmap.data_mut().as_chunks_mut::<CHANNELS>();
        for pixel in pixels {
            for channel in pixel.iter_mut().take(CHANNELS - 1) {
                *channel = (f32::from(*channel) * BACKGROUND_DIM).round() as u8;
            }
        }
    }

    fn shape(&mut self, path: &Path, fill: Option<Rgb>, outline: Option<Outline>) {
        if let Some(color) = fill {
            self.pixmap
                .fill_path(path, &paint(color), FillRule::Winding, Transform::identity(), None);
        }
        if let Some(Outline { color, width }) = outline {
            self.stroke(path, color, width);
        }
    }

    fn stroke(&mut self, path: &Path, color: Rgb, width: f32) {
        if !(width.is_finite() && width > 0.0) {
            return;
        }
        let stroke = Stroke {
            width,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        self.pixmap
            .stroke_path(path, &paint(color), &stroke, Transform::identity(), None);
    }
}

fn paint(color: Rgb) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(color.red, color.green, color.blue, u8::MAX);
    paint.anti_alias = true;
    paint
}

fn traced(points: &[Point]) -> Option<PathBuilder> {
    let (first, rest) = points.split_first()?;
    let mut builder = PathBuilder::new();
    builder.move_to(first.x, first.y);
    for point in rest {
        builder.line_to(point.x, point.y);
    }
    Some(builder)
}

fn last_index(extent: u32) -> i32 {
    i32::try_from(extent).unwrap_or(i32::MAX) - 1
}

fn coordinate(pixel: i32) -> f32 {
    pixel as f32
}
