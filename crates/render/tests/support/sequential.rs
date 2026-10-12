use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::NonZeroUsize;
use std::path::Path;

use media::decode::{Decoder, Threads};
use media::frame::{ColorRange, YuvFrame};
use types::frame::FrameIndex;
use types::timeline::Timeline;

#[derive(Debug)]
pub(crate) struct Sequential {
    pub(crate) timeline: Timeline,
    pub(crate) keys: Vec<FrameIndex>,
    pub(crate) errors: Vec<FrameIndex>,
    pub(crate) fingerprints: Vec<u64>,
}

#[cfg(test)]
pub(crate) fn fingerprint(frame: &YuvFrame) -> u64 {
    let mut hasher = DefaultHasher::new();
    (frame.y(), frame.u(), frame.v()).hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
pub(crate) fn read(path: &Path) -> Sequential {
    let mut decoder = Decoder::open(path, Threads::Exactly(NonZeroUsize::MIN)).unwrap();
    let stream = *decoder.stream();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let (mut pts, mut keys, mut errors, mut fingerprints) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        pts.push(info.pts());
        if info.is_key() {
            keys.push(info.index());
        }
        if !info.decode_errors().is_empty() {
            errors.push(info.index());
        }
        fingerprints.push(fingerprint(&frame));
    }
    Sequential {
        timeline: Timeline::new(pts, stream.time_base(), stream.frame_rate()).unwrap(),
        keys,
        errors,
        fingerprints,
    }
}
