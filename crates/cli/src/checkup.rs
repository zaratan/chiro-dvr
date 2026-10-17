use std::path::{Path, PathBuf};

use media::encode::Codec;
use media::encode::fallback::{self, Fallback};
use types::rational::Rational;

use crate::access::Access;
use crate::disk;
use crate::space::GIGABYTE;
use crate::writable::writing_refused;

#[cfg(test)]
mod tests;

pub const ENOUGH_FREE_BYTES: u64 = 20 * GIGABYTE;
const TRIAL_WIDTH: u32 = 1440;
const TRIAL_HEIGHT: u32 = 1080;
const TRIAL_FRAMES_PER_SECOND: i32 = 30;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoding {
    Preferred(Codec),
    Fallback,
    Untried,
    Broken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Space {
    Enough(u64),
    Short(u64),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checkup {
    pub folder: PathBuf,
    pub encoding: Encoding,
    pub space: Space,
    pub refused_writing: Option<Access>,
}

impl Checkup {
    pub fn problems(&self) -> usize {
        [
            matches!(self.encoding, Encoding::Broken | Encoding::Untried),
            matches!(self.space, Space::Short(_)),
            self.refused_writing.is_some(),
        ]
        .into_iter()
        .filter(|&problem| problem)
        .count()
    }
}

pub fn checkup(folder: &Path) -> Checkup {
    Checkup {
        folder: folder.to_path_buf(),
        encoding: encoding(Fallback::for_this_platform()),
        space: space(disk::volume(folder).map(|volume| volume.free)),
        refused_writing: writing_refused(folder),
    }
}

fn encoding(fallback: Fallback) -> Encoding {
    let Some(preferred) = fallback.codecs().next() else {
        return Encoding::Broken;
    };
    let Ok(rate) = Rational::new(TRIAL_FRAMES_PER_SECOND, 1) else {
        return Encoding::Broken;
    };
    match fallback.trial(TRIAL_WIDTH, TRIAL_HEIGHT, rate) {
        Ok(working) => match working.codecs().next() {
            Some(used) if used == preferred => Encoding::Preferred(used),
            Some(_) => Encoding::Fallback,
            None => Encoding::Broken,
        },
        Err(fallback::Error::Scratch(_) | fallback::Error::Destination(_)) => Encoding::Untried,
        Err(_) => Encoding::Broken,
    }
}

fn space(free: Option<u64>) -> Space {
    match free {
        Some(bytes) if bytes >= ENOUGH_FREE_BYTES => Space::Enough(bytes),
        Some(bytes) => Space::Short(bytes),
        None => Space::Unknown,
    }
}
