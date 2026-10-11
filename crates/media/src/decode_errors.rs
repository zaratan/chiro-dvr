use std::ffi::c_int;

use ffmpeg::ffi::{
    FF_DECODE_ERROR_CONCEALMENT_ACTIVE, FF_DECODE_ERROR_DECODE_SLICES, FF_DECODE_ERROR_INVALID_BITSTREAM,
    FF_DECODE_ERROR_MISSING_REFERENCE,
};
use ffmpeg_next as ffmpeg;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct DecodeErrors(c_int);

impl DecodeErrors {
    pub(crate) fn from_bits(bits: c_int) -> Self {
        Self(bits)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn invalid_bitstream(self) -> bool {
        self.has(FF_DECODE_ERROR_INVALID_BITSTREAM)
    }

    pub fn missing_reference(self) -> bool {
        self.has(FF_DECODE_ERROR_MISSING_REFERENCE)
    }

    pub fn concealment_active(self) -> bool {
        self.has(FF_DECODE_ERROR_CONCEALMENT_ACTIVE)
    }

    pub fn slices_missing(self) -> bool {
        self.has(FF_DECODE_ERROR_DECODE_SLICES)
    }

    /// The raw `decode_error_flags` bits, unknown ones included.
    pub fn bits(self) -> u32 {
        self.0.cast_unsigned()
    }

    fn has(self, flag: c_int) -> bool {
        self.0 & flag != 0
    }
}
