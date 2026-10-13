use std::num::NonZeroU32;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorRange {
    Limited,
    Full,
    Unspecified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YuvFrame {
    width: NonZeroU32,
    height: NonZeroU32,
    range: ColorRange,
    luma_row: usize,
    chroma_row: usize,
    y: Vec<u8>,
    u: Vec<u8>,
    v: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("a frame cannot be {width}×{height}")]
    EmptySize { width: u32, height: u32 },
    #[error("a {width}×{height} frame does not fit in memory")]
    TooLarge { width: u32, height: u32 },
}

impl YuvFrame {
    pub fn new(width: u32, height: u32, range: ColorRange) -> Result<Self, Error> {
        let mut frame = Self {
            width: NonZeroU32::MIN,
            height: NonZeroU32::MIN,
            range,
            luma_row: 0,
            chroma_row: 0,
            y: Vec::new(),
            u: Vec::new(),
            v: Vec::new(),
        };
        frame.reshape(width, height, range)?;
        Ok(frame)
    }

    pub(crate) fn reshape(&mut self, width: u32, height: u32, range: ColorRange) -> Result<(), Error> {
        let (Some(nonzero_width), Some(nonzero_height)) = (NonZeroU32::new(width), NonZeroU32::new(height)) else {
            return Err(Error::EmptySize { width, height });
        };
        let too_large = Error::TooLarge { width, height };
        let (luma_row, luma) = plane(width, height).ok_or(too_large)?;
        let (chroma_row, chroma) = plane(width.div_ceil(2), height.div_ceil(2)).ok_or(too_large)?;
        let reserved = reserve(&mut self.y, luma)
            .and_then(|()| reserve(&mut self.u, chroma))
            .and_then(|()| reserve(&mut self.v, chroma));
        if reserved.is_none() {
            return Err(too_large);
        }
        self.y.resize(luma, 0);
        self.u.resize(chroma, 0);
        self.v.resize(chroma, 0);
        (self.width, self.height, self.range) = (nonzero_width, nonzero_height, range);
        (self.luma_row, self.chroma_row) = (luma_row, chroma_row);
        Ok(())
    }

    pub fn copy_from(&mut self, source: &Self) -> Result<(), Error> {
        self.reshape(source.width(), source.height(), source.range)?;
        self.y.copy_from_slice(&source.y);
        self.u.copy_from_slice(&source.u);
        self.v.copy_from_slice(&source.v);
        Ok(())
    }

    pub fn width(&self) -> u32 {
        self.width.get()
    }

    pub fn height(&self) -> u32 {
        self.height.get()
    }

    pub fn range(&self) -> ColorRange {
        self.range
    }

    pub fn y(&self) -> &[u8] {
        &self.y
    }

    pub fn u(&self) -> &[u8] {
        &self.u
    }

    pub fn v(&self) -> &[u8] {
        &self.v
    }

    pub fn y_mut(&mut self) -> &mut [u8] {
        &mut self.y
    }

    pub fn u_mut(&mut self) -> &mut [u8] {
        &mut self.u
    }

    pub fn v_mut(&mut self) -> &mut [u8] {
        &mut self.v
    }

    pub(crate) fn planes(&self) -> [(&[u8], usize); 3] {
        [
            (&self.y, self.luma_row),
            (&self.u, self.chroma_row),
            (&self.v, self.chroma_row),
        ]
    }

    pub(crate) fn planes_mut(&mut self) -> [(&mut [u8], usize); 3] {
        [
            (&mut self.y, self.luma_row),
            (&mut self.u, self.chroma_row),
            (&mut self.v, self.chroma_row),
        ]
    }
}

fn plane(width: u32, height: u32) -> Option<(usize, usize)> {
    let row = usize::try_from(width).ok()?;
    Some((row, row.checked_mul(usize::try_from(height).ok()?)?))
}

fn reserve(plane: &mut Vec<u8>, length: usize) -> Option<()> {
    plane.try_reserve_exact(length.saturating_sub(plane.len())).ok()
}
