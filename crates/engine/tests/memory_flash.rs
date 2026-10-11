use std::path::{Path, PathBuf};

use engine::config::DetectConfig;
use engine::detection::detect;
use media::decode::Threads;
use media::encode::{Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use rayon::ThreadPoolBuilder;
use tempfile::TempDir;
use types::rational::Rational;

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

const WIDTH: u32 = 1440;
const HEIGHT: u32 = 1080;
const WORK_PIXELS: usize = 960 * 720;
const WORKERS: usize = 4;
const MOST_BYTES_PER_WORK_PIXEL_AND_WORKER: usize = 32;
const LEAST_BYTES_PER_WORK_PIXEL_AND_WORKER: usize = 16;
const FRAMES: usize = 64;
const FLASHES: [usize; WORKERS] = [1, 6, 11, 16];
const BLOCK: usize = 48;

#[cfg(test)]
fn encoded(path: &Path, flashes: &[usize]) -> PathBuf {
    let settings = Settings::new(WIDTH, HEIGHT, Rational::new(30, 1).unwrap(), Codec::OpenH264).unwrap();
    let mut encoder = Encoder::create(path, &settings).unwrap();
    let mut frame = YuvFrame::new(WIDTH, HEIGHT, ColorRange::Limited).unwrap();
    frame.u_mut().fill(128);
    frame.v_mut().fill(128);
    for index in 0..FRAMES {
        for (pixel, luma) in frame.y_mut().iter_mut().enumerate() {
            let (column, row) = (pixel % WIDTH as usize, pixel / WIDTH as usize);
            let block = column / BLOCK + row / BLOCK + index;
            *luma = match (flashes.contains(&index), block % 2) {
                (false, _) => 120,
                (true, 0) => 60,
                (true, _) => 180,
            };
        }
        encoder.push(&frame).unwrap();
    }
    encoder.finish().unwrap();
    path.to_path_buf()
}

#[test]
fn a_full_frame_flash_seen_by_every_worker_costs_at_most_32_bytes_per_work_pixel_and_worker() {
    let folder = TempDir::new().unwrap();
    let calm = encoded(&folder.path().join("calm.mp4"), &[]);
    let flash = encoded(&folder.path().join("flash.mp4"), &FLASHES);
    let pool = ThreadPoolBuilder::new().num_threads(WORKERS).build().unwrap();
    let profiler = dhat::Profiler::builder().testing().build();

    let quiet = pool.install(|| detect(&calm, Threads::Automatic, &DetectConfig::default()).unwrap());
    let calm_peak = dhat::HeapStats::get().max_bytes;
    drop(quiet);
    let flashed = pool.install(|| detect(&flash, Threads::Automatic, &DetectConfig::default()).unwrap());
    let flash_peak = dhat::HeapStats::get().max_bytes;
    drop(profiler);

    let extra = flash_peak.saturating_sub(calm_peak);
    assert_eq!(flashed.detections().frame_count(), FRAMES);
    assert!(
        extra > LEAST_BYTES_PER_WORK_PIXEL_AND_WORKER * WORK_PIXELS * WORKERS,
        "{extra}"
    );
    assert!(
        extra <= MOST_BYTES_PER_WORK_PIXEL_AND_WORKER * WORK_PIXELS * WORKERS,
        "{extra} bytes, {} per work pixel and worker",
        extra / WORK_PIXELS / WORKERS
    );
}
