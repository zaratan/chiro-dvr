use draw::color::Rgb;
use types::track::Track;

use crate::ink::Ink;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{colors} colours were given for {tracks} tracks")]
pub struct Mismatch {
    pub colors: usize,
    pub tracks: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Colored<'a> {
    tracks: &'a [Track],
    colors: &'a [Option<Rgb>],
}

impl<'a> Colored<'a> {
    pub fn new(tracks: &'a [Track], colors: &'a [Option<Rgb>]) -> Result<Self, Mismatch> {
        if colors.is_empty() || colors.len() == tracks.len() {
            Ok(Self { tracks, colors })
        } else {
            Err(Mismatch {
                colors: colors.len(),
                tracks: tracks.len(),
            })
        }
    }

    #[must_use]
    pub const fn tracks(&self) -> &'a [Track] {
        self.tracks
    }

    pub(crate) fn inks(&self) -> Vec<Ink> {
        (0..self.tracks.len())
            .map(|index| Ink::of(self.colors.get(index).copied().flatten()))
            .collect()
    }
}
