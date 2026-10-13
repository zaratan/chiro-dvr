use std::num::{NonZeroU32, NonZeroUsize};

use types::detection::Detection;
use types::size::SourceSize;

use rayon::ThreadPoolBuilder;

use super::{BATCH, Detector, gain_of};
use crate::blur::Blur;
use crate::closing::Disk;
use crate::config::DetectConfig;
use crate::geometry::Geometry;
use crate::mask::Mask;
use crate::settings::DetectSettings;
use crate::synthetic::{Noise, background, luma, with_square};

const WIDTH: usize = 360;
const HEIGHT: usize = 180;
const GRAY: f64 = 200.0;
const NOISE: f64 = 3.0;

fn detected(frames: impl IntoIterator<Item = Vec<u8>>) -> Vec<Vec<Detection>> {
    detected_by(frames, BATCH)
}

fn detected_by(frames: impl IntoIterator<Item = Vec<u8>>, batch: NonZeroUsize) -> Vec<Vec<Detection>> {
    detected_with(frames, batch, 15, &DetectConfig::default())
}

fn detected_with(
    frames: impl IntoIterator<Item = Vec<u8>>,
    batch: NonZeroUsize,
    half: usize,
    config: &DetectConfig,
) -> Vec<Vec<Detection>> {
    let source = SourceSize::new(
        NonZeroU32::new(WIDTH as u32).unwrap(),
        NonZeroU32::new(HEIGHT as u32).unwrap(),
    );
    let geometry = Geometry::new(source, NonZeroU32::new(WIDTH as u32 / 3 * 2).unwrap()).unwrap();
    let blur = Blur::new(1.0, geometry).unwrap();
    let disk = Disk::new(4.0, geometry);
    let unmasked = Mask::new(&[], geometry.width(), geometry.height(), 0).unwrap();
    let mut detector = Detector::new(
        geometry,
        &blur,
        &disk,
        unmasked,
        NonZeroUsize::new(half).unwrap(),
        config,
        batch,
    );
    for frame in frames {
        detector.push(&frame, |_| {});
    }
    detector.finish().iter().map(|(_, found)| found.to_vec()).collect()
}

fn calm(index: usize) -> Vec<u8> {
    background(WIDTH, HEIGHT, GRAY, NOISE, index as u64)
}

fn moving_square(count: usize, size: usize, delta: f64) -> impl Iterator<Item = Vec<u8>> {
    (0..count).map(move |index| with_square(calm(index), WIDTH, (30 + 6 * index, 90), size, delta))
}

fn single_at(found: &[Vec<Detection>], x: impl Fn(usize) -> f64, y: f64) {
    for (index, frame) in found.iter().enumerate() {
        assert_eq!(frame.len(), 1, "frame {index}: {frame:?}");
        assert!((frame[0].x - x(index)).abs() < 2.0, "frame {index}: {frame:?}");
        assert!((frame[0].y - y).abs() < 2.0, "frame {index}: {frame:?}");
    }
}

#[test]
fn a_dark_moving_spot_is_detected_on_every_frame_within_two_source_pixels_of_its_centre() {
    let found = detected(moving_square(40, 6, -80.0));

    assert_eq!(found.len(), 40);
    single_at(&found, |index| 32.5 + 6.0 * index as f64, 92.5);
    assert!(
        found
            .iter()
            .enumerate()
            .all(|(index, frame)| frame[0].frame.get() == index)
    );
}

#[test]
fn a_bright_moving_spot_is_detected_too() {
    let found = detected(moving_square(40, 6, 50.0));

    single_at(&found, |index| 32.5 + 6.0 * index as f64, 92.5);
}

#[test]
fn a_global_gain_flickering_from_frame_to_frame_is_compensated() {
    let mut noise = Noise::new(5);
    let frames: Vec<Vec<u8>> = (0..40)
        .map(|index| background(WIDTH, HEIGHT, GRAY + 20.0 * noise.uniform() - 10.0, NOISE, index))
        .collect();

    let found = detected(frames);

    assert!(found.iter().all(Vec::is_empty), "{found:?}");
}

#[test]
fn a_video_shorter_than_the_background_window_is_still_analysed() {
    let found = detected(moving_square(12, 6, -80.0));

    assert_eq!(found.len(), 12);
    single_at(&found, |index| 32.5 + 6.0 * index as f64, 92.5);
}

#[test]
fn no_frames_give_no_detections() {
    assert_eq!(detected([]), Vec::<Vec<Detection>>::new());
}

#[test]
fn fragments_closer_than_the_merge_radius_become_one_blob() {
    let frames = (0..40).map(|index| {
        let left = with_square(calm(index), WIDTH, (30 + 6 * index, 90), 6, -80.0);
        with_square(left, WIDTH, (39 + 6 * index, 90), 6, -80.0)
    });

    let found = detected(frames);

    for frame in &found {
        assert_eq!(frame.len(), 1, "{frame:?}");
        assert!(frame[0].width >= 15.0, "{frame:?}");
    }
}

fn flickering(index: usize, noise: &mut Noise) -> Vec<u8> {
    let mut frame = calm(index);
    for row in (30..150).step_by(12) {
        for column in (150..210).step_by(12) {
            frame = with_square(frame, WIDTH, (column, row), 6, 15.0 * noise.gaussian());
        }
    }
    with_square(frame, WIDTH, (6 + 3 * index, 45), 6, -30.0)
}

#[test]
fn flickering_pixels_raise_their_own_threshold_where_the_window_holds_seven_samples() {
    let mut noise = Noise::new(11);
    let frames: Vec<Vec<u8>> = (0..40).map(|index| flickering(index, &mut noise)).collect();

    let found = detected(frames);

    let in_strip = |frame: &Vec<Detection>| frame.iter().filter(|found| found.x > 140.0).count() as f64;
    let fixed_floor = [0, 1, 2, 37, 38, 39];
    let (fixed, raised): (Vec<_>, Vec<_>) = found
        .iter()
        .enumerate()
        .partition(|(index, _)| fixed_floor.contains(index));
    let per_frame = |frames: &[(usize, &Vec<Detection>)]| {
        frames.iter().map(|(_, frame)| in_strip(frame)).sum::<f64>() / frames.len() as f64
    };
    assert!(
        per_frame(&raised) < per_frame(&fixed) / 4.0,
        "{} {}",
        per_frame(&raised),
        per_frame(&fixed)
    );
    for (index, frame) in found.iter().enumerate() {
        let spot: Vec<&Detection> = frame.iter().filter(|found| found.x < 140.0).collect();
        assert_eq!(spot.len(), 1, "frame {index}: {frame:?}");
        assert!((spot[0].y - 47.5).abs() < 2.0, "frame {index}: {frame:?}");
    }
}

#[test]
fn single_pixel_noise_is_smoothed_away_while_a_target_sized_spot_survives() {
    let mut noise = Noise::new(13);
    let frames = (0..40).map(|index| {
        let mut frame = vec![luma(120.0); WIDTH * HEIGHT];
        for _ in 0..30 {
            let pixel = (noise.uniform() * (WIDTH * HEIGHT) as f64) as usize;
            let row_start = pixel / WIDTH * WIDTH;
            let column = 90 + pixel % (WIDTH - 90);
            let sign = if noise.uniform() < 0.5 { -40.0 } else { 40.0 };
            frame = with_square(frame, WIDTH, (column, row_start / WIDTH), 1, sign);
        }
        with_square(frame, WIDTH, (6 + 2 * index, 45), 6, -30.0)
    });

    let found = detected(frames);

    single_at(&found, |index| 8.5 + 2.0 * index as f64, 47.5);
}

#[test]
fn the_amplitude_is_measured_on_the_filtered_residual_in_gray_levels() {
    let found = detected(moving_square(40, 3, -80.0));

    let amplitudes: Vec<f64> = found.iter().flatten().map(|found| found.amplitude).collect();
    assert_eq!(amplitudes.len(), 40);
    assert!(
        amplitudes
            .iter()
            .all(|&amplitude| (12.0..0.8 * 80.0).contains(&amplitude)),
        "{amplitudes:?}"
    );
}

#[test]
fn a_gain_is_the_distance_of_a_level_to_the_mean_level_in_units() {
    assert!((gain_of(130, 300, 3, 10) - 3.0).abs() < f32::EPSILON);
    assert!((gain_of(70, 300, 3, 10) + 3.0).abs() < f32::EPSILON);
}

#[test]
fn the_detections_are_the_same_bit_for_bit_whatever_the_threads_and_the_batch() {
    let frames: Vec<Vec<u8>> = {
        let mut noise = Noise::new(11);
        (0..40).map(|index| flickering(index, &mut noise)).collect()
    };
    let run = |threads: usize, batch: usize| {
        ThreadPoolBuilder::new()
            .num_threads(threads)
            .build()
            .unwrap()
            .install(|| detected_by(frames.clone(), NonZeroUsize::new(batch).unwrap()))
    };

    let one_by_one = run(1, 1);

    assert!(one_by_one.iter().map(Vec::len).sum::<usize>() > 40);
    assert_eq!(run(4, 16), one_by_one);
    assert_eq!(run(3, 7), one_by_one);
    assert_eq!(run(4, 64), one_by_one);
}

#[test]
fn sixty_one_and_eighty_one_background_samples_give_the_same_detections_bit_for_bit_whatever_the_threads() {
    let every_frame = DetectConfig::try_from(&DetectSettings {
        bg_step: 1,
        ..DetectSettings::default()
    })
    .unwrap();
    let frames: Vec<Vec<u8>> = {
        let mut noise = Noise::new(13);
        (0..100)
            .map(|index| with_square(flickering(index, &mut noise), WIDTH, (30 + 3 * index, 90), 6, -80.0))
            .collect()
    };
    for half in [30, 40] {
        let run = |threads: usize, batch: usize| {
            ThreadPoolBuilder::new()
                .num_threads(threads)
                .build()
                .unwrap()
                .install(|| detected_with(frames.clone(), NonZeroUsize::new(batch).unwrap(), half, &every_frame))
        };

        let one_by_one = run(1, 1);

        assert!(one_by_one.iter().map(Vec::len).sum::<usize>() >= 100, "{half}");
        assert_eq!(run(4, 16), one_by_one, "{half}");
        assert_eq!(run(3, 7), one_by_one, "{half}");
    }
}
