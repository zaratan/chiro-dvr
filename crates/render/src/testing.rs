use std::io;
use std::num::NonZeroU32;
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use media::encode::{self, Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use types::detection::Detection;
use types::frame::FrameIndex;
use types::rate::FrameRate;
use types::rational::Rational;
use types::track::Track;

use crate::encoders::{Open, Request, Sink};

pub(crate) const FRAMES: usize = 48;
const WIDTH: u32 = 1440;
const HEIGHT: u32 = 1080;
const KEY_INTERVAL: u32 = 8;

pub(crate) fn binoculars(folder: &Path) -> PathBuf {
    binoculars_of(folder, WIDTH, HEIGHT)
}

pub(crate) fn binoculars_of(folder: &Path, width: u32, height: u32) -> PathBuf {
    let path = folder.join("binoculars.mp4");
    let settings = Settings::new(width, height, Rational::new(10_000, 333).unwrap(), Codec::OpenH264)
        .unwrap()
        .with_key_interval(NonZeroU32::new(KEY_INTERVAL).unwrap());
    let mut encoder = Encoder::create(&path, &settings).unwrap();
    let mut frame = YuvFrame::new(width, height, ColorRange::Limited).unwrap();
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    for index in 0..FRAMES {
        for (row, line) in frame.y_mut().chunks_mut(width as usize).enumerate() {
            for (column, value) in line.iter_mut().enumerate() {
                let texture = ((column + 7 * index) ^ (row + 3 * index)) % 200;
                *value = 16 + u8::try_from(texture).unwrap();
            }
        }
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
    path
}

pub(crate) fn losing_frames(folder: &Path, parts: Range<usize>, count: usize) -> PathBuf {
    let path = binoculars(folder);
    media::payload::wipe(&path, parts, count).unwrap();
    path
}

pub(crate) fn track(id: usize, points: &[(usize, f64, f64)]) -> Track {
    let detections = points
        .iter()
        .map(|&(frame, x, y)| Detection {
            frame: FrameIndex::new(frame),
            x,
            y,
            left: x - 5.0,
            top: y - 5.0,
            width: 10.0,
            height: 10.0,
            area: 100.0,
            amplitude: 40.0,
        })
        .collect();
    Track::new(id, detections).unwrap()
}

pub(crate) fn thirty() -> FrameRate {
    FrameRate::new(Rational::new(30, 1).unwrap()).unwrap()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Breaking {
    Open,
    PushAt(usize),
    Persist,
}

#[derive(Debug, Default)]
pub(crate) struct Fake {
    pub(crate) breaks: Vec<(&'static str, Codec, Breaking)>,
    live_hardware: Arc<AtomicUsize>,
    most_hardware: Arc<AtomicUsize>,
    opened: Mutex<Vec<(String, Codec)>>,
}

#[derive(Debug)]
struct Live(Arc<AtomicUsize>);

impl Drop for Live {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Debug)]
pub(crate) struct FakeSink {
    encoder: Encoder,
    path: PathBuf,
    pushed: usize,
    breaking: Option<Breaking>,
    _live: Option<Live>,
}

impl Fake {
    pub(crate) fn breaking(breaks: Vec<(&'static str, Codec, Breaking)>) -> Self {
        Self {
            breaks,
            ..Self::default()
        }
    }

    pub(crate) fn most_hardware_at_once(&self) -> usize {
        self.most_hardware.load(Ordering::SeqCst)
    }

    pub(crate) fn opened(&self) -> Vec<(String, Codec)> {
        self.opened.lock().unwrap().clone()
    }
}

impl Open for Fake {
    type Sink = FakeSink;

    fn open(&self, request: &Request<'_>) -> Result<FakeSink, encode::Error> {
        let name = request.path.file_name().unwrap().to_string_lossy().into_owned();
        let breaking = self
            .breaks
            .iter()
            .find(|(broken, codec, _)| *broken == name && *codec == request.codec)
            .map(|(_, _, breaking)| *breaking);
        if breaking == Some(Breaking::Open) {
            return Err(encode::Error::EncoderMissing { name: "forced" });
        }
        let software = Request {
            codec: Codec::OpenH264,
            ..*request
        };
        let encoder = Encoder::create(request.path, &software.settings()?)?;
        let live = (request.codec == Codec::VideoToolbox).then(|| {
            let now = self.live_hardware.fetch_add(1, Ordering::SeqCst) + 1;
            self.most_hardware.fetch_max(now, Ordering::SeqCst);
            Live(Arc::clone(&self.live_hardware))
        });
        self.opened.lock().unwrap().push((name, request.codec));
        Ok(FakeSink {
            encoder,
            path: request.path.to_path_buf(),
            pushed: 0,
            breaking,
            _live: live,
        })
    }
}

impl Sink for FakeSink {
    fn push(&mut self, frame: &YuvFrame) -> Result<(), encode::Error> {
        if self.breaking == Some(Breaking::PushAt(self.pushed)) {
            return Err(encode::Error::FrameLayout);
        }
        self.pushed += 1;
        self.encoder.push(frame)
    }

    fn finish(self) -> Result<(), encode::Error> {
        if self.breaking == Some(Breaking::Persist) {
            return Err(encode::Error::Persist {
                path: self.path,
                source: io::Error::other("forced"),
            });
        }
        self.encoder.finish()
    }
}
