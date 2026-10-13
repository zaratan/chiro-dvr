#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("a luma plane of {length} pixels is not a whole number of rows of {width} pixels")]
pub struct Error {
    pub length: usize,
    pub width: usize,
}

#[derive(Debug)]
pub struct LumaMut<'a> {
    pixels: &'a mut [u8],
    width: usize,
}

impl<'a> LumaMut<'a> {
    pub fn new(pixels: &'a mut [u8], width: usize) -> Result<Self, Error> {
        let length = pixels.len();
        if width == 0 || length == 0 || !length.is_multiple_of(width) {
            return Err(Error { length, width });
        }
        Ok(Self { pixels, width })
    }

    pub(crate) fn of_frame(pixels: &'a mut [u8], width: usize) -> Self {
        Self { pixels, width }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.pixels.len() / self.width
    }

    pub fn rows_mut(&mut self) -> std::slice::ChunksExactMut<'_, u8> {
        self.pixels.chunks_exact_mut(self.width)
    }
}
