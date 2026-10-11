use std::num::NonZeroU32;

use crate::size::SourceSize;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorkScale {
    source: SourceSize,
    work_width: NonZeroU32,
    work_height: NonZeroU32,
}

impl WorkScale {
    pub fn new(source: SourceSize, requested_width: NonZeroU32) -> Self {
        let work_width = requested_width.min(source.width());
        let height = f64::from(work_width.get()) * f64::from(source.height().get()) / f64::from(source.width().get());
        let work_height = NonZeroU32::new(height.round() as u32).unwrap_or(NonZeroU32::MIN);
        Self {
            source,
            work_width,
            work_height,
        }
    }

    pub fn source(self) -> SourceSize {
        self.source
    }

    pub fn work_width(self) -> NonZeroU32 {
        self.work_width
    }

    pub fn work_height(self) -> NonZeroU32 {
        self.work_height
    }

    pub fn factor(self) -> f64 {
        f64::from(self.source.width().get()) / f64::from(self.work_width.get())
    }

    pub fn coordinate_to_source(self, work: f64) -> f64 {
        (work + 0.5) * self.factor() - 0.5
    }

    pub fn length_to_source(self, work: f64) -> f64 {
        work * self.factor()
    }

    pub fn area_to_source(self, work: f64) -> f64 {
        work * self.factor().powi(2)
    }
}
