#![cfg(target_os = "macos")]

use media::decode::Decoder;
use media::encode::{Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use types::rational::Rational;

#[test]
#[ignore = "hardware"]
fn videotoolbox_without_software_fallback_encodes_one_second() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("hardware.mp4");
    let codec = Codec::VideoToolbox { allow_software: false };
    let settings = Settings::new(1440, 1080, Rational::new(10_000, 333).unwrap(), codec).unwrap();
    let mut encoder = Encoder::create(&path, &settings).unwrap();
    let mut frame = YuvFrame::new(1440, 1080, ColorRange::Limited).unwrap();
    for index in 0..30_u8 {
        frame.y_mut().fill(16 + index * 7);
        frame.u_mut().fill(128);
        frame.v_mut().fill(128);
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
    let mut decoder = Decoder::open(&path).unwrap();
    let mut frames = 0;
    while decoder.next(&mut frame).unwrap().is_some() {
        frames += 1;
    }
    assert_eq!(frames, 30);
}
