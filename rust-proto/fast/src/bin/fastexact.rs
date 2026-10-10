//! Single-pass detection from the luma plane, with OpenCV's and numpy's arithmetic (float32 blur,
//! numpy means, float64 residual): the 8-bit work gray is OpenCV's on every neutral pixel.

use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Instant;

use batdetect_core::frame::{Blob, BlobConfig, bgr_to_gray, find_blobs_sparse, gaussian_blur_f32_par, mean_f32, resize_area_bgr};
use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::{Context as Scaler, Flags};
use std::cell::RefCell;
use batdetect_core::pipeline::{DetectConfig, MIN_NOISE_SAMPLES, mean_f64};
use batdetect_core::{FrameParams, residual_and_threshold};
use fast::prep::{area_bgr_two_thirds, luma_to_gray, work_size};
use ffmpeg::util::frame::video::Video;
use ffmpeg_next as ffmpeg;
use rayon::prelude::*;

const BATCH: usize = 32;
const KERNEL_BITS: [u32; 9] =
    [0x390c54e2, 0x3b913926, 0x3d5d25cd, 0x3e77c75d, 0x3ecc4252, 0x3e77c75d, 0x3d5d25cd, 0x3b913926, 0x390c54e2];

thread_local! {
    static SCALER: RefCell<Option<Scaler>> = const { RefCell::new(None) };
}

/// OpenCV's reading: the decoded YUV 4:2:0 frame to BGR24 by swscale, bicubic, default colour matrix.
fn opencv_bgr(planes: &[Vec<u8>; 3], width: usize, height: usize) -> Vec<u8> {
    let mut yuv = Video::new(Pixel::YUV420P, width as u32, height as u32);
    for (p, plane) in planes.iter().enumerate() {
        let (w, h) = if p == 0 { (width, height) } else { (width.div_ceil(2), height.div_ceil(2)) };
        let stride = yuv.stride(p);
        let data = yuv.data_mut(p);
        for row in 0..h {
            data[row * stride..row * stride + w].copy_from_slice(&plane[row * w..(row + 1) * w]);
        }
    }
    SCALER.with(|cell| {
        let mut slot = cell.borrow_mut();
        let scaler = slot.get_or_insert_with(|| {
            Scaler::get(Pixel::YUV420P, width as u32, height as u32, Pixel::BGR24, width as u32, height as u32, Flags::BICUBIC)
                .expect("scaler")
        });
        let mut bgr = Video::empty();
        scaler.run(&yuv, &mut bgr).expect("scale");
        let stride = bgr.stride(0);
        let mut packed = Vec::with_capacity(width * height * 3);
        for row in 0..height {
            packed.extend_from_slice(&bgr.data(0)[row * stride..row * stride + width * 3]);
        }
        packed
    })
}

struct Work {
    index: usize,
    data: Vec<f32>,
    level: f64,
}

fn detect(window: &VecDeque<Arc<Work>>, target: usize, last: usize, half: usize, cfg: &DetectConfig, size: (usize, usize), scale: f64, mask: &[bool]) -> Vec<Blob> {
    let base = window[0].index;
    let first = target.saturating_sub(half);
    let stop = (target + half).min(last);
    let sampled: Vec<&Arc<Work>> = (first..=stop).step_by(cfg.bg_step).map(|i| &window[i - base]).collect();
    let levels: Vec<f64> = sampled.iter().map(|w| w.level).collect();
    let mean_level = mean_f64(&levels);
    let gains: Vec<f64> = levels.iter().map(|l| l - mean_level).collect();
    let current = &window[target - base];
    let params = FrameParams {
        gain_offset: current.level - mean_level,
        gains: &gains,
        threshold: cfg.threshold,
        noise_factor: cfg.noise_factor,
        use_noise: cfg.noise_factor > 0.0 && sampled.len() >= MIN_NOISE_SAMPLES,
    };
    let frames: Vec<&[f32]> = sampled.iter().map(|w| w.data.as_slice()).collect();
    let n = size.0 * size.1;
    let (mut residual, mut thresholds) = (vec![0f64; n], vec![0f64; n]);
    if !residual_and_threshold(&frames, &current.data, mask, &params, &mut residual, &mut thresholds, false) {
        thresholds.iter_mut().for_each(|t| *t = cfg.threshold);
    }
    let blob_cfg = BlobConfig { min_area: cfg.min_area, max_area: cfg.max_area, merge_radius: cfg.merge_radius };
    find_blobs_sparse(&residual, &thresholds, size.0, size.1, &blob_cfg, scale)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let (input, detections_path, frames_path) = (args[1].clone(), &args[2], &args[3]);
    let started = Instant::now();
    ffmpeg::init()?;
    ffmpeg::log::set_level(ffmpeg::log::Level::Quiet);
    let ictx = ffmpeg::format::input(&input)?;
    let stream = ictx.streams().best(ffmpeg::media::Type::Video).ok_or("no video stream")?;
    let rate = stream.rate();
    let fps = f64::from(rate.numerator()) / f64::from(rate.denominator());
    let index = stream.index();
    let mut context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
    context.set_threading(ffmpeg::threading::Config { kind: ffmpeg::threading::Type::Frame, count: 0 });
    let decoder = context.decoder().video()?;
    let (width, height) = (decoder.width() as usize, decoder.height() as usize);
    let (work_w, work_h) = work_size(width, height).ok_or("frame size must divide by 3")?;
    let scale = width as f64 / work_w as f64;
    let kernel: Vec<f32> = KERNEL_BITS.iter().map(|&b| f32::from_bits(b)).collect();
    let cfg = DetectConfig::default();
    let half = cfg.half_window(fps);
    let mask = vec![true; work_w * work_h];
    let exact_bgr = env::var("FAST_EXACT_BGR").is_ok_and(|v| v == "1");
    let (tx, rx) = sync_channel::<([Vec<u8>; 3], Option<i64>, bool, i32)>(96);
    let reader = thread::spawn(move || {
        let (mut ictx, mut decoder) = (ictx, decoder);
        let drain = |decoder: &mut ffmpeg::decoder::Video| -> bool {
            let mut frame = Video::empty();
            while decoder.receive_frame(&mut frame).is_ok() {
                let plane = |p: usize, w: usize, h: usize| {
                    let (stride, data) = (frame.stride(p), frame.data(p));
                    let mut out = Vec::with_capacity(w * h);
                    for row in 0..h {
                        out.extend_from_slice(&data[row * stride..row * stride + w]);
                    }
                    out
                };
                let planes = if exact_bgr {
                    let (cw, ch) = (width.div_ceil(2), height.div_ceil(2));
                    [plane(0, width, height), plane(1, cw, ch), plane(2, cw, ch)]
                } else {
                    [plane(0, width, height), Vec::new(), Vec::new()]
                };
                let flags = unsafe { (*frame.as_ptr()).decode_error_flags };
                if tx.send((planes, frame.pts(), frame.is_key(), flags)).is_err() {
                    return false;
                }
            }
            true
        };
        for (s, packet) in ictx.packets() {
            if s.index() == index {
                let _ = decoder.send_packet(&packet);
                if !drain(&mut decoder) {
                    return;
                }
            }
        }
        let _ = decoder.send_eof();
        drain(&mut decoder);
    });
    let mut frames_out = BufWriter::new(File::create(frames_path)?);
    let mut window: VecDeque<Arc<Work>> = VecDeque::new();
    let mut results: Vec<(usize, Vec<Blob>)> = Vec::new();
    let (mut count, mut next_target, mut finished) = (0usize, 0usize, false);
    while !finished {
        let mut batch = Vec::with_capacity(BATCH);
        while batch.len() < BATCH {
            match rx.recv() {
                Ok(item) => batch.push(item),
                Err(_) => {
                    finished = true;
                    break;
                }
            }
        }
        for (k, (_, pts, key, flags)) in batch.iter().enumerate() {
            let pts = pts.map_or("-".to_string(), |p| p.to_string());
            writeln!(frames_out, "{} {pts} {} 0 {flags}", count + k, u8::from(*key))?;
        }
        let base = count;
        let works: Vec<Work> = batch
            .par_iter()
            .enumerate()
            .map(|(k, (planes, ..))| {
                let mut gray = Vec::new();
                if exact_bgr {
                    let bgr = opencv_bgr(planes, width, height);
                    let small = if env::var("FAST_GENERIC_AREA").is_ok() {
                        resize_area_bgr(&bgr, width, height, work_w, work_h, false)
                    } else {
                        let mut small = Vec::new();
                        area_bgr_two_thirds(&bgr, width, height, &mut small);
                        small
                    };
                    gray = bgr_to_gray(&small);
                } else {
                    luma_to_gray(&planes[0], width, width, height, &mut gray);
                }
                let data = gaussian_blur_f32_par(&gray, work_w, work_h, &kernel);
                let level = mean_f32(&data);
                Work { index: base + k, data, level }
            })
            .collect();
        count += works.len();
        window.extend(works.into_iter().map(Arc::new));
        let Some(last) = window.back().map(|w| w.index) else { break };
        let ready = if finished { last + 1 } else { (last + 1).saturating_sub(half) };
        let targets: Vec<usize> = (next_target..ready).collect();
        results.extend(targets.par_iter().map(|&t| (t, detect(&window, t, last, half, &cfg, (work_w, work_h), scale, &mask))).collect::<Vec<_>>());
        next_target = next_target.max(ready);
        let keep = next_target.saturating_sub(half);
        while window.front().is_some_and(|w| w.index < keep) {
            window.pop_front();
        }
    }
    reader.join().expect("reader");
    frames_out.flush()?;
    let mut out = BufWriter::new(File::create(detections_path)?);
    let mut total = 0;
    for (frame, blobs) in &results {
        for b in blobs {
            total += 1;
            writeln!(out, "{frame} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}", b.x, b.y, b.left, b.top, b.width, b.height, b.area, b.amplitude)?;
        }
    }
    out.flush()?;
    eprintln!("{count} frames, {total} detections, {:.2} s total", started.elapsed().as_secs_f64());
    Ok(())
}
