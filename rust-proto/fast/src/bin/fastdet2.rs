//! Detection in parallel time slices, each with its own decoder (VideoToolbox or software),
//! numbered by the sample index of the mp4. Same output files as `fastdet`.

use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ptr;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;

use fast::prep::{prepare, work_size};
use fast::{Config, Detector, Work};
use ffmpeg::ffi;
use ffmpeg::util::frame::video::Video;
use ffmpeg_next as ffmpeg;
use rayon::prelude::*;

const BATCH: usize = 24;

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
}

struct Sample {
    timestamp: i64,
    key: bool,
}

fn samples(ictx: &ffmpeg::format::context::Input, index: usize) -> Vec<Sample> {
    let stream = ictx.stream(index).expect("video stream");
    unsafe {
        let st = stream.as_ptr() as *mut ffi::AVStream;
        let count = ffi::avformat_index_get_entries_count(st);
        (0..count)
            .map(|k| {
                let e = &*ffi::avformat_index_get_entry(st, k);
                Sample { timestamp: e.timestamp, key: e.flags() & ffi::AVINDEX_KEYFRAME as i32 != 0 }
            })
            .collect()
    }
}

unsafe extern "C" fn pick_videotoolbox(
    _ctx: *mut ffi::AVCodecContext,
    formats: *const ffi::AVPixelFormat,
) -> ffi::AVPixelFormat {
    let mut p = formats;
    unsafe {
        while *p != ffi::AVPixelFormat::AV_PIX_FMT_NONE {
            if *p == ffi::AVPixelFormat::AV_PIX_FMT_VIDEOTOOLBOX {
                return *p;
            }
            p = p.add(1);
        }
        *formats
    }
}

struct FrameInfo {
    pts: Option<i64>,
    key: bool,
    error_flags: i32,
}

fn flush(
    batch: &mut Vec<(usize, Vec<u8>)>,
    finished: bool,
    detector: &mut Detector,
    found: &mut Vec<(usize, Vec<fast::blobs::Blob>)>,
    size: (usize, usize),
) -> f64 {
    let t = Instant::now();
    let (width, height) = size;
    let (work_w, work_h) = work_size(width, height).expect("size");
    let works: Vec<Work> = batch
        .par_iter()
        .map(|(k, luma)| {
            let mut data = Vec::new();
            let sum = prepare(luma, width, width, height, &mut data);
            Work { index: *k, data, level: sum as f64 / (work_w * work_h) as f64 }
        })
        .collect();
    batch.clear();
    found.extend(detector.push(works, finished));
    t.elapsed().as_secs_f64()
}

struct SliceResult {
    aligned: bool,
    offset: Option<i64>,
    found: Vec<(usize, Vec<fast::blobs::Blob>)>,
    frames: Vec<(usize, FrameInfo)>,
    decode_s: f64,
    detect_s: f64,
}

#[allow(clippy::too_many_arguments)]
fn run_slice(
    input: &str,
    hardware: bool,
    fps: f64,
    size: (usize, usize),
    targets: std::ops::Range<usize>,
    last_frame: usize,
    start_key: usize,
    start_ts: i64,
    timestamps: std::sync::Arc<Vec<i64>>,
) -> Result<SliceResult, ffmpeg::Error> {
    let (width, height) = size;
    let (work_w, work_h) = work_size(width, height).expect("size");
    let scale = width as f64 / work_w as f64;
    let mut detector = Detector::new(Config::default(), fps, (work_w, work_h), scale, targets.clone(), Some(last_frame));
    let (need_from, need_until) = (detector.first_needed(), detector.last_needed());
    let mut ictx = ffmpeg::format::input(&input)?;
    let index = ictx.streams().best(ffmpeg::media::Type::Video).ok_or(ffmpeg::Error::StreamNotFound)?.index();
    let params = ictx.stream(index).unwrap().parameters();
    let mut context = ffmpeg::codec::context::Context::from_parameters(params)?;
    if hardware {
        unsafe {
            let mut device: *mut ffi::AVBufferRef = ptr::null_mut();
            let rc = ffi::av_hwdevice_ctx_create(
                &mut device,
                ffi::AVHWDeviceType::AV_HWDEVICE_TYPE_VIDEOTOOLBOX,
                ptr::null(),
                ptr::null_mut(),
                0,
            );
            if rc < 0 {
                return Err(ffmpeg::Error::from(rc));
            }
            let c = context.as_mut_ptr();
            (*c).hw_device_ctx = ffi::av_buffer_ref(device);
            (*c).get_format = Some(pick_videotoolbox);
            ffi::av_buffer_unref(&mut device);
        }
    } else {
        context.set_threading(ffmpeg::threading::Config {
            kind: ffmpeg::threading::Type::Frame,
            count: env_usize("FAST_DECODE_THREADS", 0),
        });
    }
    let mut decoder = context.decoder().video()?;
    unsafe {
        let rc = ffi::av_seek_frame(ictx.as_mut_ptr(), index as i32, start_ts, ffi::AVSEEK_FLAG_BACKWARD);
        if rc < 0 {
            return Err(ffmpeg::Error::from(rc));
        }
    }
    let (tx, rx) = std::sync::mpsc::sync_channel::<(usize, Vec<u8>, Option<FrameInfo>)>(2 * BATCH);
    let targets_in = targets.clone();
    let reader = thread::spawn(move || -> Result<(f64, bool, Option<i64>), ffmpeg::Error> {
        let t0 = Instant::now();
        let mut next = start_key;
        let (mut aligned, mut offset) = (true, None::<i64>);
        let mut frame = Video::empty();
        let mut packets = ictx.packets();
        let (mut done, mut eof) = (false, false);
        loop {
            while !done && decoder.receive_frame(&mut frame).is_ok() {
                let k = next;
                next += 1;
                match (frame.pts(), timestamps.get(k)) {
                    (Some(p), Some(&ts)) => {
                        let o = *offset.get_or_insert(p - ts);
                        aligned &= p - ts == o;
                    }
                    _ => aligned = false,
                }
                let info = targets_in.contains(&k).then(|| FrameInfo {
                    pts: frame.pts(),
                    key: frame.is_key(),
                    error_flags: unsafe { (*frame.as_ptr()).decode_error_flags },
                });
                let mut luma = Vec::new();
                if k >= need_from {
                    let mut sw = Video::empty();
                    let (data, stride) = if hardware {
                        let rc = unsafe { ffi::av_hwframe_transfer_data(sw.as_mut_ptr(), frame.as_ptr(), 0) };
                        if rc < 0 {
                            return Err(ffmpeg::Error::from(rc));
                        }
                        (sw.data(0), sw.stride(0))
                    } else {
                        (frame.data(0), frame.stride(0))
                    };
                    luma.reserve_exact(width * height);
                    for row in 0..height {
                        luma.extend_from_slice(&data[row * stride..row * stride + width]);
                    }
                }
                if tx.send((k, luma, info)).is_err() {
                    return Ok((t0.elapsed().as_secs_f64(), aligned, offset));
                }
                done = k >= need_until;
            }
            if done || eof {
                break;
            }
            match packets.next() {
                Some((s, packet)) => {
                    if s.index() == index {
                        let _ = decoder.send_packet(&packet);
                    }
                }
                None => {
                    let _ = decoder.send_eof();
                    eof = true;
                }
            }
        }
        Ok((t0.elapsed().as_secs_f64(), aligned && done, offset))
    });
    let mut frames = Vec::new();
    let mut batch: Vec<(usize, Vec<u8>)> = Vec::with_capacity(BATCH);
    let mut found = Vec::new();
    let mut detect_s = 0f64;
    let mut reached_end = false;
    for (k, luma, info) in rx {
        if let Some(info) = info {
            frames.push((k, info));
        }
        if k >= need_from {
            batch.push((k, luma));
        }
        reached_end = k >= need_until;
        if batch.len() == BATCH || reached_end {
            detect_s += flush(&mut batch, false, &mut detector, &mut found, size);
        }
    }
    let (decode_s, aligned, offset) = reader.join().expect("reader")?;
    if !reached_end {
        detect_s += flush(&mut batch, true, &mut detector, &mut found, size);
    }
    Ok(SliceResult { aligned, offset, found, frames, decode_s, detect_s })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let (input, detections_path, frames_path) = (args[1].clone(), &args[2], &args[3]);
    let hardware = env::var("FAST_DECODER").is_ok_and(|v| v == "videotoolbox");
    let slices = env_usize("FAST_SLICES", 8);
    let started = Instant::now();
    ffmpeg::init()?;
    ffmpeg::log::set_level(ffmpeg::log::Level::Quiet);
    let ictx = ffmpeg::format::input(&input)?;
    let stream = ictx.streams().best(ffmpeg::media::Type::Video).ok_or("no video stream")?;
    let rate = stream.rate();
    let fps = f64::from(rate.numerator()) / f64::from(rate.denominator());
    let index = stream.index();
    let params = stream.parameters();
    let probe = ffmpeg::codec::context::Context::from_parameters(params)?.decoder().video()?;
    let size = (probe.width() as usize, probe.height() as usize);
    let table = samples(&ictx, index);
    let total = table.len();
    let keys: Vec<usize> = (0..total).filter(|&k| table[k].key).collect();
    let timestamps = std::sync::Arc::new(table.iter().map(|t| t.timestamp).collect::<Vec<i64>>());
    let half = Config::default().half_window(fps);
    let bounds: Vec<usize> = (0..=slices).map(|s| s * total / slices).collect();
    let results: Mutex<Vec<(usize, SliceResult)>> = Mutex::new(Vec::new());
    thread::scope(|scope| {
        for s in 0..slices {
            let (from, to) = (bounds[s], bounds[s + 1]);
            if from == to {
                continue;
            }
            let need = from.saturating_sub(half);
            let key = *keys.iter().rev().find(|&&k| k <= need).unwrap_or(&0);
            let (input, results, ts, stamps) = (&input, &results, table[key].timestamp, timestamps.clone());
            scope.spawn(move || {
                let r = run_slice(input, hardware, fps, size, from..to, total - 1, key, ts, stamps).expect("slice failed");
                results.lock().unwrap().push((s, r));
            });
        }
    });
    let mut results = results.into_inner().unwrap();
    results.sort_by_key(|r| r.0);
    let offsets: std::collections::BTreeSet<Option<i64>> = results.iter().map(|r| r.1.offset).collect();
    if results.iter().any(|r| !r.1.aligned) || offsets.len() != 1 {
        eprintln!("frames do not follow the mp4 samples one to one (lost or reordered frames): slices are unsafe");
        std::process::exit(3);
    }
    let mut frames_out = BufWriter::new(File::create(frames_path)?);
    let mut out = BufWriter::new(File::create(detections_path)?);
    let cfg = Config::default();
    let (mut count, mut detections, mut decode_s, mut detect_s) = (0usize, 0usize, 0f64, 0f64);
    let mut expected = 0usize;
    for (_, r) in &results {
        decode_s += r.decode_s;
        detect_s += r.detect_s;
        for (k, info) in &r.frames {
            if *k != expected {
                return Err(format!("frame {expected} missing, got {k}").into());
            }
            expected += 1;
            let pts = info.pts.map_or("-".to_string(), |p| p.to_string());
            writeln!(frames_out, "{k} {pts} {} 0 {}", u8::from(info.key), info.error_flags)?;
            count += 1;
        }
        let mut found = r.found.clone();
        found.sort_by_key(|f| f.0);
        for (frame, blobs) in &found {
            for b in blobs {
                detections += 1;
                writeln!(
                    out,
                    "{frame} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
                    b.x,
                    b.y,
                    b.left,
                    b.top,
                    b.width,
                    b.height,
                    b.area,
                    cfg.units_to_gray(b.peak)
                )?;
            }
        }
    }
    frames_out.flush()?;
    out.flush()?;
    eprintln!(
        "{count} frames of {total} samples, {detections} detections, {:.2} s total, {} slices, {} decoder; summed per slice: decode {decode_s:.2} s, detect {detect_s:.2} s",
        started.elapsed().as_secs_f64(),
        slices,
        if hardware { "videotoolbox" } else { "software" }
    );
    Ok(())
}
