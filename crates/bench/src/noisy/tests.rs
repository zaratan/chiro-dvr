use std::num::NonZeroUsize;

use media::decode::{Decoder, Threads};
use media::frame::{ColorRange, YuvFrame};
use tempfile::TempDir;
use types::frame::FrameIndex;

use super::{FRAMES, HEIGHT, NOISE_HIGH, NOISE_LOW, WIDTH, slow_target, write_video};

const DECODING_MARGIN: f64 = 4.0;

#[test]
fn the_noisy_video_decodes_at_its_size_and_length_within_a_few_levels_of_its_written_noise() {
    let folder = TempDir::new().unwrap();
    let path = folder.path().join("noisy.mp4");
    write_video(&path).unwrap();
    let band = (NOISE_LOW - DECODING_MARGIN)..=(NOISE_HIGH + DECODING_MARGIN);

    let mut decoder = Decoder::open(&path, Threads::Exactly(NonZeroUsize::MIN)).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut frames = 0;
    while decoder.next(&mut frame).unwrap().is_some() {
        assert_eq!((frame.width(), frame.height()), (WIDTH, HEIGHT));
        assert!(frame.y().iter().all(|&luma| band.contains(&f64::from(luma))));
        frames += 1;
    }
    assert_eq!(frames, FRAMES);
}

#[test]
fn the_slow_target_of_the_tests_stays_inside_the_noisy_video() {
    let target = slow_target(0, 10, 40).unwrap();

    assert!(target.end() < FrameIndex::new(FRAMES));
    assert!(
        target
            .positions()
            .iter()
            .all(|point| { (0.0..f64::from(WIDTH)).contains(&point.x) && (0.0..f64::from(HEIGHT)).contains(&point.y) })
    );
}
