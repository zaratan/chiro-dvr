use media::decode::Decoder;
use media::encode::Codec;
use media::encode::fallback::Fallback;
use media::frame::{ColorRange, YuvFrame};
use types::rational::Rational;

pub(crate) const FIRST: usize = 660;
pub(crate) const FRAMES: usize = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Stream {
    pub(crate) frames: usize,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) frame_rate: Rational,
    pub(crate) range: ColorRange,
    pub(crate) codec: Codec,
}

fn skipped(frame: &mut YuvFrame) -> Decoder {
    let source = reference::fixture::fixture("video_092_original_3m24-4m05.mp4").unwrap();
    let mut decoder = Decoder::open(&source).unwrap();
    for _ in 0..FIRST {
        decoder.next(frame).unwrap().unwrap();
    }
    decoder
}

pub(crate) fn round_trip(fallback: &Fallback) -> (Stream, f64) {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("passage.mp4");
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut original = skipped(&mut frame);
    let source = *original.stream();
    let mut encoder = fallback
        .create(&path, source.width(), source.height(), source.frame_rate())
        .unwrap();
    let codec = encoder.codec();
    for _ in 0..FRAMES {
        original.next(&mut frame).unwrap().unwrap();
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
    let mut original = skipped(&mut frame);
    let mut reread = Decoder::open(&path).unwrap();
    let written = *reread.stream();
    let mut copy = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let (mut frames, mut squared, mut pixels) = (0, 0_u64, 0_u64);
    while reread.next(&mut copy).unwrap().is_some() {
        original.next(&mut frame).unwrap().unwrap();
        let errors = frame
            .y()
            .iter()
            .zip(copy.y())
            .map(|(a, b)| u64::from(a.abs_diff(*b)).pow(2));
        squared += errors.sum::<u64>();
        pixels += u64::try_from(copy.y().len()).unwrap();
        frames += 1;
    }
    let stream = Stream {
        frames,
        width: written.width(),
        height: written.height(),
        frame_rate: written.frame_rate(),
        range: written.range(),
        codec,
    };
    let luma_psnr = 10.0 * (255.0 * 255.0 * pixels as f64 / squared as f64).log10();
    (stream, luma_psnr)
}

pub(crate) fn expected(codec: Codec) -> Stream {
    Stream {
        frames: FRAMES,
        width: 1440,
        height: 1080,
        frame_rate: Rational::new(10_000, 333).unwrap(),
        range: ColorRange::Limited,
        codec,
    }
}
