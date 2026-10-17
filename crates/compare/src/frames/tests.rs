use std::path::Path;

use engine::config::DetectConfig;
use engine::detection::detect;
use media::decode::Threads;
use media::encode::{Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use reference::frames::Frames;
use tempfile::TempDir;
use types::rational::Rational;

use super::{frames_of, pass_of};

const FRAMES: usize = 8;

fn encoded(path: &Path) {
    let settings = Settings::new(1440, 1080, Rational::new(30, 1).unwrap(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(path, &settings).unwrap();
    let mut frame = YuvFrame::new(1440, 1080, ColorRange::Limited).unwrap();
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    for index in 0..FRAMES {
        frame.y_mut().fill(100 + index as u8);
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
}

#[test]
fn every_decoded_frame_gets_a_line_with_its_timestamp_its_key_flag_and_its_detections() {
    let folder = TempDir::new().unwrap();
    let video = folder.path().join("flat.mp4");
    encoded(&video);
    let detected = detect(&video, Threads::Automatic, &DetectConfig::default()).unwrap();

    let frames = frames_of(&detected).unwrap();

    let entries = frames.entries();
    assert_eq!(entries.len(), FRAMES);
    assert!(entries[0].key_frame);
    assert!(entries.iter().all(|entry| !entry.decode_error && entry.pts.is_some()));
    assert!(entries.windows(2).all(|pair| pair[0].pts < pair[1].pts));
    assert_eq!(frames.detections(), detected.detections());
}

#[test]
fn the_frames_written_read_back_as_the_same_table() {
    let folder = TempDir::new().unwrap();
    let video = folder.path().join("flat.mp4");
    encoded(&video);
    let frames = frames_of(&detect(&video, Threads::Automatic, &DetectConfig::default()).unwrap()).unwrap();
    let path = folder.path().join("frames.jsonl");

    frames.write(&path).unwrap();

    assert_eq!(Frames::read(&path).unwrap(), frames);
}

#[test]
fn a_pass_keeps_the_frames_and_the_damage_its_reading_reported() {
    let folder = TempDir::new().unwrap();
    let video = folder.path().join("wiped.mp4");
    encoded(&video);
    media::payload::wipe(&video, 1..2, 4).unwrap();
    let detected = detect(&video, Threads::Automatic, &DetectConfig::default()).unwrap();

    let pass = pass_of(&detected).unwrap();

    assert_ne!(pass.reports.error_frames.len() + pass.reports.jump_frames.len(), 0);
    assert_eq!(&pass.reports, detected.damage().reports());
    assert_eq!(pass.frames, frames_of(&detected).unwrap());
}
