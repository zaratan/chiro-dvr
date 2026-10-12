use std::path::{Path, PathBuf};

use engine::config::DetectConfig;
use engine::detection::detect;
use media::decode::Threads;
use media::encode::{Codec, Encoder, Settings};
use media::frame::{ColorRange, YuvFrame};
use rayon::ThreadPoolBuilder;
use tempfile::TempDir;
use types::rational::Rational;
use types::scale::WorkScale;
use types::size::SourceSize;

pub(crate) const WIDTH: u32 = 1440;
pub(crate) const HEIGHT: u32 = 1080;
const WORKERS: usize = 4;
const MOST_BYTES_PER_WORK_PIXEL_AND_WORKER: usize = 32;
const LEAST_BYTES_PER_WORK_PIXEL_AND_WORKER: usize = 16;
const FRAMES: usize = 64;
const FLASHES: [usize; WORKERS] = [1, 6, 11, 16];
const BLOCK: usize = 48;

pub(crate) struct Flash {
    pub(crate) extra_bytes: usize,
    pub(crate) work_pixels: usize,
    pub(crate) frames: usize,
}

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

#[cfg(test)]
pub(crate) fn flash_cost(config: &DetectConfig) -> Flash {
    let folder = TempDir::new().unwrap();
    let calm = encoded(&folder.path().join("calm.mp4"), &[]);
    let flash = encoded(&folder.path().join("flash.mp4"), &FLASHES);
    let pool = ThreadPoolBuilder::new().num_threads(WORKERS).build().unwrap();
    let profiler = dhat::Profiler::builder().testing().build();

    let quiet = pool.install(|| detect(&calm, Threads::Automatic, config).unwrap());
    let calm_peak = dhat::HeapStats::get().max_bytes;
    drop(quiet);
    let flashed = pool.install(|| detect(&flash, Threads::Automatic, config).unwrap());
    let flash_peak = dhat::HeapStats::get().max_bytes;
    drop(profiler);

    let source = SourceSize::new(WIDTH.try_into().unwrap(), HEIGHT.try_into().unwrap());
    let scale = WorkScale::new(source, config.work_width());
    Flash {
        extra_bytes: flash_peak.saturating_sub(calm_peak),
        work_pixels: (scale.work_width().get() * scale.work_height().get()) as usize,
        frames: flashed.detections().frame_count(),
    }
}

#[cfg(test)]
pub(crate) fn assert_within_bounds(cost: &Flash) {
    assert_eq!(cost.frames, FRAMES);
    assert!(
        cost.extra_bytes > LEAST_BYTES_PER_WORK_PIXEL_AND_WORKER * cost.work_pixels * WORKERS,
        "{}",
        cost.extra_bytes
    );
    assert!(
        cost.extra_bytes <= MOST_BYTES_PER_WORK_PIXEL_AND_WORKER * cost.work_pixels * WORKERS,
        "{} bytes, {} per work pixel and worker",
        cost.extra_bytes,
        cost.extra_bytes / cost.work_pixels / WORKERS
    );
}
