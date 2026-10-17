use std::path::Path;
use std::sync::OnceLock;

use engine::config::DetectConfig;
use engine::detection::detect;
use media::decode::Threads;
use media::encode::{Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use tempfile::TempDir;
use tracking::exclusion::exclude;
use tracking::stability::StabilityConfig;
use tracking::track::{TrackConfig, track_detections};
use types::rate::FrameRate;
use types::rational::Rational;
use types::span::Span;

const WIDTH: usize = 1440;
const HEIGHT: usize = 1080;
const RATE: i32 = 30;
const JUMP_FRAME: usize = 45;
const GRAY: f64 = 200.0;
const NOISE: f64 = 3.0;
const NOISE_SHIFT: usize = 7919;
const JUMPS: [u8; 3] = [20, 30, 40];

struct Outcome {
    detections: usize,
    unstable: Vec<Span>,
    tracks: usize,
}

fn gaussian_table() -> Vec<f64> {
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut uniform = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        (state >> 11) as f64 / (1_u64 << 53) as f64
    };
    (0..WIDTH * HEIGHT + 2 * JUMP_FRAME * NOISE_SHIFT)
        .map(|_| (0..12).map(|_| uniform()).sum::<f64>() - 6.0)
        .collect()
}

fn luma(gray: f64) -> u8 {
    (gray * 219.0 / 255.0 + 16.0).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
fn encode_jump(path: &Path, jump: u8, noise: &[f64]) {
    let settings = Settings::new(
        WIDTH as u32,
        HEIGHT as u32,
        Rational::new(RATE, 1).unwrap(),
        Codec::OpenH264,
    )
    .unwrap();
    let mut encoder = Encoder::create(path, &settings).unwrap();
    let mut frame = YuvFrame::new(WIDTH as u32, HEIGHT as u32, ColorRange::Limited).unwrap();
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    for index in 0..2 * JUMP_FRAME {
        let level = if index < JUMP_FRAME {
            GRAY
        } else {
            GRAY + f64::from(jump)
        };
        let shifted = &noise[index * NOISE_SHIFT..];
        for (value, noise) in frame.y_mut().iter_mut().zip(shifted) {
            *value = luma(level + NOISE * noise);
        }
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
}

#[cfg(test)]
fn outcome(jump: u8) -> &'static Outcome {
    static OUTCOMES: OnceLock<Vec<Outcome>> = OnceLock::new();
    let outcomes = OUTCOMES.get_or_init(|| {
        let folder = TempDir::new().unwrap();
        let noise = gaussian_table();
        let rate = FrameRate::new(Rational::new(RATE, 1).unwrap()).unwrap();
        JUMPS
            .iter()
            .map(|&jump| {
                let path = folder.path().join(format!("jump_{jump}.mp4"));
                encode_jump(&path, jump, &noise);
                let detected = detect(&path, Threads::Automatic, &DetectConfig::default()).unwrap();
                let exclusion = exclude(detected.detections(), Vec::new(), 0, rate, StabilityConfig::default());
                let tracks = track_detections(&exclusion, detected.reading().video().size(), TrackConfig::default())
                    .unwrap()
                    .len();
                Outcome {
                    detections: detected.detections().detection_count(),
                    unstable: exclusion.unstable,
                    tracks,
                }
            })
            .collect()
    });
    &outcomes[JUMPS.iter().position(|&known| known == jump).unwrap()]
}

#[test]
#[ignore = "slow"]
fn a_global_brightness_jump_of_20_30_or_40_levels_leaves_no_track_with_the_default_settings() {
    for jump in JUMPS {
        assert_eq!(outcome(jump).tracks, 0, "jump of {jump}");
    }
}

#[test]
#[ignore = "slow"]
fn a_large_brightness_jump_is_ignored_around_the_jump_rather_than_tracked() {
    let jump = outcome(30);

    assert!(jump.detections > 0);
    assert_ne!(jump.unstable, Vec::new());
    assert!(jump.unstable.iter().all(|span| {
        span.first().get() <= JUMP_FRAME + RATE as usize / 2 && span.last().get() >= JUMP_FRAME - RATE as usize / 2
    }));
    assert!(
        jump.unstable
            .iter()
            .all(|span| span.first().get() > 0 && span.last().get() < 2 * JUMP_FRAME - 1)
    );
}

#[test]
#[ignore = "slow"]
fn a_moderate_brightness_jump_lights_nothing_at_the_real_scale_and_ignores_nothing() {
    let jump = outcome(20);

    assert_eq!(jump.detections, 0);
    assert_eq!(jump.unstable, Vec::new());
}
