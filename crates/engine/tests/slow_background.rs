use std::path::Path;

use engine::config::DetectConfig;
use engine::detection::detect_input;
use engine::input::Input;
use media::decode::{Packet, Threads};
use media::encode::{Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use types::rational::Rational;

const WIDTH: u32 = 1440;
const HEIGHT: u32 = 1080;
const DARK: u8 = 67;
const BRIGHT: u8 = 205;
const FRAMES: usize = 90;
const FIRST_BRIGHT_FRAME: usize = 25;
const FIRST_UNREADABLE_PACKET: usize = 60;

#[cfg(test)]
fn dark_then_bright(path: &Path) {
    let settings = Settings::new(WIDTH, HEIGHT, Rational::new(30, 1).unwrap(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(path, &settings).unwrap();
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    for index in 0..FRAMES {
        frame
            .y_mut()
            .fill(if index < FIRST_BRIGHT_FRAME { DARK } else { BRIGHT });
        frame.u_mut().fill(128);
        frame.v_mut().fill(128);
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
}

#[test]
#[ignore = "slow"]
fn the_background_leaves_out_the_frames_of_a_damaged_span_even_when_they_are_the_majority() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("unreadable_end.mp4");
    dark_then_bright(&path);
    let unreadable_end = |packet: &mut Packet<'_>| {
        if packet.ordinal() >= FIRST_UNREADABLE_PACKET {
            packet.data_mut().unwrap().fill(0xff);
        }
    };
    let input = Input::open(&path, Threads::Automatic)
        .unwrap()
        .with_seam(Box::new(unreadable_end));

    let detected = detect_input(input, &DetectConfig::default()).unwrap();

    assert_ne!(detected.damage().spans().len(), 0);
    let level = detected.background().unwrap().y()[0];
    assert!(level.abs_diff(DARK) <= 3, "{level}");
}
