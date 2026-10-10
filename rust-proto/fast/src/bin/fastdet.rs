use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Instant;

use fast::prep::{prepare, work_size};
use fast::{Config, Detector, Work};
use ffmpeg::format::Pixel;
use ffmpeg::util::frame::video::Video;
use ffmpeg_next as ffmpeg;
use rayon::prelude::*;

const BATCH: usize = 48;
const QUEUE: usize = 96;

struct Decoded {
    luma: Vec<u8>,
    pts: Option<i64>,
    key: bool,
    corrupt: bool,
    error_flags: i32,
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
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
    context.set_threading(ffmpeg::threading::Config {
        kind: ffmpeg::threading::Type::Frame,
        count: env_usize("FAST_DECODE_THREADS", 0),
    });
    if env::var("FAST_GRAY").is_ok_and(|v| v == "1") {
        unsafe { (*context.as_mut_ptr()).flags |= ffmpeg::ffi::AV_CODEC_FLAG_GRAY as i32 };
    }
    let decoder = context.decoder().video()?;
    let (width, height) = (decoder.width() as usize, decoder.height() as usize);
    let (work_w, work_h) = work_size(width, height).ok_or("frame size must divide by 3")?;
    let scale = width as f64 / work_w as f64;
    let (tx, rx) = sync_channel::<Decoded>(QUEUE);
    let reader = thread::spawn(move || -> Result<f64, ffmpeg::Error> {
        let (mut ictx, mut decoder) = (ictx, decoder);
        let t0 = Instant::now();
        let drain = |decoder: &mut ffmpeg::decoder::Video| -> bool {
            let mut frame = Video::empty();
            while decoder.receive_frame(&mut frame).is_ok() {
                if !matches!(frame.format(), Pixel::YUV420P | Pixel::YUVJ420P) {
                    eprintln!("unsupported pixel format {:?}: only 8-bit 4:2:0 video is read", frame.format());
                    std::process::exit(2);
                }
                let stride = frame.stride(0);
                let data = frame.data(0);
                let mut luma = Vec::with_capacity(width * height);
                for row in 0..height {
                    luma.extend_from_slice(&data[row * stride..row * stride + width]);
                }
                let error_flags = unsafe { (*frame.as_ptr()).decode_error_flags };
                let item = Decoded { luma, pts: frame.pts(), key: frame.is_key(), corrupt: frame.is_corrupt(), error_flags };
                if tx.send(item).is_err() {
                    return false;
                }
            }
            true
        };
        for (s, packet) in ictx.packets() {
            if s.index() == index {
                let _ = decoder.send_packet(&packet);
                if !drain(&mut decoder) {
                    return Ok(t0.elapsed().as_secs_f64());
                }
            }
        }
        decoder.send_eof()?;
        drain(&mut decoder);
        Ok(t0.elapsed().as_secs_f64())
    });

    let mut detector = Detector::new(Config::default(), fps, (work_w, work_h), scale, 0..usize::MAX, None);
    let mut frames_out = BufWriter::new(File::create(frames_path)?);
    let mut results = Vec::new();
    let (mut prep_s, mut detect_s, mut wait_s) = (0f64, 0f64, 0f64);
    let mut count = 0usize;
    let mut finished = false;
    while !finished {
        let t = Instant::now();
        let mut batch = Vec::with_capacity(BATCH);
        while batch.len() < BATCH {
            match rx.recv() {
                Ok(d) => batch.push(d),
                Err(_) => {
                    finished = true;
                    break;
                }
            }
        }
        wait_s += t.elapsed().as_secs_f64();
        for (k, d) in batch.iter().enumerate() {
            let pts = d.pts.map_or("-".to_string(), |p| p.to_string());
            writeln!(frames_out, "{} {} {} {} {}", count + k, pts, u8::from(d.key), u8::from(d.corrupt), d.error_flags)?;
        }
        let t = Instant::now();
        let base = count;
        let works: Vec<Work> = batch
            .par_iter()
            .enumerate()
            .map(|(k, d)| {
                let mut data = Vec::new();
                let sum = prepare(&d.luma, width, width, height, &mut data);
                Work { index: base + k, data, level: sum as f64 / (work_w * work_h) as f64 }
            })
            .collect();
        count += works.len();
        prep_s += t.elapsed().as_secs_f64();
        let t = Instant::now();
        results.extend(detector.push(works, finished));
        detect_s += t.elapsed().as_secs_f64();
    }
    let decode_s = reader.join().expect("reader thread")?;
    frames_out.flush()?;
    results.sort_by_key(|r| r.0);
    let mut out = BufWriter::new(File::create(detections_path)?);
    let mut total = 0;
    for (frame, blobs) in &results {
        for b in blobs {
            total += 1;
            let amplitude = detector.cfg.units_to_gray(b.peak);
            writeln!(
                out,
                "{frame} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
                b.x, b.y, b.left, b.top, b.width, b.height, b.area, amplitude
            )?;
        }
    }
    out.flush()?;
    eprintln!(
        "{count} frames, {} targets, {total} detections, {:.2} s total; decode thread {decode_s:.2} s, waiting {wait_s:.2} s, prep {prep_s:.2} s, detect {detect_s:.2} s, fps {fps}",
        results.len(),
        started.elapsed().as_secs_f64()
    );
    Ok(())
}
