use skrifa::charmap::Charmap;
use skrifa::instance::{Location, LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlineGlyphCollection, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};
use tiny_skia::{Path, PathBuilder};
use unicode_normalization::UnicodeNormalization;

#[cfg(test)]
mod tests;

const ROBOTO: &[u8] = include_bytes!("../fonts/Roboto.ttf");
const REGULAR: f32 = 400.0;
const SEMIBOLD: f32 = 600.0;
const REFERENCE_SIZE: f32 = 100.0;
const REPLACEMENT: char = '\u{FFFD}';

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("the embedded font cannot be read")]
    Font(#[source] skrifa::raw::ReadError),
    #[error("the embedded font has no outline for the digit zero")]
    NoDigit,
    #[error("digits cannot be {0} pixels tall")]
    DigitSize(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weight {
    Regular,
    SemiBold,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextStyle {
    digit_px: f32,
    weight: Weight,
}

impl TextStyle {
    pub fn new(digit_px: f32, weight: Weight) -> Result<Self, Error> {
        if digit_px.is_finite() && digit_px > 0.0 {
            Ok(Self { digit_px, weight })
        } else {
            Err(Error::DigitSize(digit_px))
        }
    }

    pub(crate) const fn trusted(digit_px: f32, weight: Weight) -> Self {
        Self { digit_px, weight }
    }

    pub fn digit_px(self) -> f32 {
        self.digit_px
    }

    pub fn weight(self) -> Weight {
        self.weight
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapedText {
    pub(crate) glyphs: Vec<(Path, f32)>,
    width: f32,
}

impl ShapedText {
    pub fn width(&self) -> f32 {
        self.width
    }
}

pub struct Typeface {
    font: FontRef<'static>,
    charmap: Charmap<'static>,
    outlines: OutlineGlyphCollection<'static>,
    regular: Location,
    semibold: Location,
    size_per_digit_px: f32,
}

impl std::fmt::Debug for Typeface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Typeface")
            .field("size_per_digit_px", &self.size_per_digit_px)
            .finish_non_exhaustive()
    }
}

struct Pen(PathBuilder);

impl OutlinePen for Pen {
    fn move_to(&mut self, x: f32, y: f32) {
        self.0.move_to(x, -y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.0.line_to(x, -y);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        self.0.quad_to(cx0, -cy0, x, -y);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.0.cubic_to(cx0, -cy0, cx1, -cy1, x, -y);
    }

    fn close(&mut self) {
        self.0.close();
    }
}

impl Typeface {
    pub fn roboto() -> Result<Self, Error> {
        let font = FontRef::new(ROBOTO).map_err(Error::Font)?;
        let axes = font.axes();
        let mut typeface = Self {
            charmap: font.charmap(),
            outlines: font.outline_glyphs(),
            regular: axes.location([("wght", REGULAR)]),
            semibold: axes.location([("wght", SEMIBOLD)]),
            font,
            size_per_digit_px: 1.0,
        };
        let zero = typeface.outline(typeface.glyph_id('0'), REFERENCE_SIZE, Weight::Regular);
        let digit_height = zero.map(|path| path.bounds().height()).ok_or(Error::NoDigit)?;
        typeface.size_per_digit_px = REFERENCE_SIZE / digit_height;
        Ok(typeface)
    }

    pub fn measure(&self, text: &str, style: TextStyle) -> f32 {
        let metrics = self
            .font
            .glyph_metrics(Size::new(self.size(style)), self.location(style.weight));
        text.nfc()
            .map(|character| metrics.advance_width(self.glyph_id(character)).unwrap_or_default())
            .sum()
    }

    pub fn shape(&self, text: &str, style: TextStyle) -> ShapedText {
        let size = self.size(style);
        let metrics = self.font.glyph_metrics(Size::new(size), self.location(style.weight));
        let mut glyphs = Vec::new();
        let mut left = 0.0;
        for character in text.nfc() {
            let id = self.glyph_id(character);
            if let Some(path) = self.outline(id, size, style.weight) {
                glyphs.push((path, left));
            }
            left += metrics.advance_width(id).unwrap_or_default();
        }
        ShapedText { glyphs, width: left }
    }

    fn size(&self, style: TextStyle) -> f32 {
        style.digit_px * self.size_per_digit_px
    }

    fn glyph_id(&self, character: char) -> GlyphId {
        self.charmap
            .map(character)
            .or_else(|| self.charmap.map(REPLACEMENT))
            .unwrap_or_default()
    }

    fn outline(&self, id: GlyphId, size: f32, weight: Weight) -> Option<Path> {
        let glyph = self.outlines.get(id)?;
        let mut pen = Pen(PathBuilder::new());
        let settings = DrawSettings::unhinted(Size::new(size), self.location(weight));
        glyph.draw(settings, &mut pen).ok()?;
        pen.0.finish()
    }

    fn location(&self, weight: Weight) -> LocationRef<'_> {
        match weight {
            Weight::Regular => LocationRef::from(&self.regular),
            Weight::SemiBold => LocationRef::from(&self.semibold),
        }
    }
}
