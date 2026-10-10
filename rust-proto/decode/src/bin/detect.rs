use std::env;
use std::io::{BufWriter, Write};
use std::sync::mpsc::sync_channel;
use std::thread;
use std::time::Instant;

use batdetect_core::frame::{bgr_to_gray, gaussian_blur_f32, gaussian_blur_f32_par, mean_f32, resize_area_bgr, resize_area_bgr_par};
use batdetect_core::pipeline::{DetectConfig, Detector, Entry};
use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::util::frame::video::Video;
use ffmpeg_next as ffmpeg;

const READ_AHEAD: usize = 4;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let input = args[1].clone();
    let kernel: Vec<f32> = args[2].split(',').map(|v| f32::from_bits(u32::from_str_radix(v, 16).unwrap())).collect();
    let parallel = env::var("BATDETECT_RUST_THREADS").map_or(true, |v| v != "0");
    let cfg = DetectConfig::default();
    ffmpeg::init()?;
    ffmpeg::log::set_level(ffmpeg::log::Level::Quiet);
    let ictx = ffmpeg::format::input(&input)?;
    let stream = ictx.streams().best(ffmpeg::media::Type::Video).ok_or("no video stream")?;
    let rate = stream.avg_frame_rate();
    let fps = f64::from(rate.numerator()) / f64::from(rate.denominator());
    let params = stream.parameters();
    let index = stream.index();
    let mut context = ffmpeg::codec::context::Context::from_parameters(params)?;
    context.set_threading(ffmpeg::threading::Config { kind: ffmpeg::threading::Type::Frame, count: 0 });
    let decoder = context.decoder().video()?;
    let (width, height) = (decoder.width() as usize, decoder.height() as usize);
    let work_width = cfg.work_width.min(width);
    let work_height = ((work_width * height) as f64 / width as f64).round_ties_even().max(1.0) as usize;
    let scale = width as f64 / work_width as f64;
    let sigma = cfg.target_sigma / scale;
    let started = Instant::now();
    let (tx, rx) = sync_channel::<Entry>(READ_AHEAD);
    let reader = thread::spawn(move || -> Result<usize, ffmpeg::Error> {
        let (mut ictx, mut decoder) = (ictx, decoder);
        let mut scaler = Context::get(
            decoder.format(),
            width as u32,
            height as u32,
            Pixel::BGR24,
            width as u32,
            height as u32,
            Flags::BICUBIC,
        )?;
        let mut count = 0usize;
        let mut spent = [0f64; 5];
        let mut drain = |decoder: &mut ffmpeg::decoder::Video, count: &mut usize| -> Result<bool, ffmpeg::Error> {
            let mut decoded = Video::empty();
            let mut t = Instant::now();
            while decoder.receive_frame(&mut decoded).is_ok() {
                let mut lap = |k: usize| { spent[k] += t.elapsed().as_secs_f64(); t = Instant::now(); };
                let mut bgr = Video::empty();
                scaler.run(&decoded, &mut bgr)?;
                let stride = bgr.stride(0);
                let mut packed = Vec::with_capacity(width * height * 3);
                for row in 0..height {
                    packed.extend_from_slice(&bgr.data(0)[row * stride..row * stride + width * 3]);
                }
                lap(0);
                let small = if parallel { resize_area_bgr_par(&packed, width, height, work_width, work_height, false) } else { resize_area_bgr(&packed, width, height, work_width, work_height, false) };
                lap(1);
                let gray = bgr_to_gray(&small);
                let frame = if sigma > 0.0 && parallel { gaussian_blur_f32_par(&gray, work_width, work_height, &kernel) } else if sigma > 0.0 { gaussian_blur_f32(&gray, work_width, work_height, &kernel) } else {
                    gray.iter().map(|&v| f32::from(v)).collect()
                };
                lap(2);
                let level = mean_f32(&frame);
                lap(3);
                if tx.send(Entry { index: *count, frame, level }).is_err() {
                    return Ok(false);
                }
                *count += 1;
                t = Instant::now();
            }
            Ok(true)
        };
        for (s, packet) in ictx.packets() {
            if s.index() == index {
                decoder.send_packet(&packet)?;
                if !drain(&mut decoder, &mut count)? {
                    return Ok(count);
                }
            }
        }
        decoder.send_eof()?;
        drain(&mut decoder, &mut count)?;
        eprintln!("reader: decode+sws {:.2} s, resize {:.2} s, gray+blur {:.2} s, mean {:.2} s", spent[0], spent[1], spent[2], spent[3]);
        Ok(count)
    });
    let mut detector = Detector::new(&cfg, fps, work_width, work_height, scale, parallel);
    let mut results = Vec::new();
    let mut last = None;
    let mut busy = 0f64;
    for entry in rx {
        last = Some(entry.index);
        let t = Instant::now();
        if let Some(found) = detector.push(entry) {
            results.push(found);
        }
        busy += t.elapsed().as_secs_f64();
    }
    eprintln!("detector busy {busy:.2} s");
    let frames = reader.join().expect("reader thread")?;
    if let Some(last) = last {
        results.extend(detector.finish(last));
    }
    let elapsed = started.elapsed().as_secs_f64();
    let mut out = BufWriter::new(std::io::stdout().lock());
    let mut total = 0;
    for (frame, blobs) in &results {
        for b in blobs {
            total += 1;
            writeln!(
                out,
                "{frame} {:?} {:?} {:?} {:?} {:?} {:?} {:?} {:?}",
                b.x, b.y, b.left, b.top, b.width, b.height, b.area, b.amplitude
            )?;
        }
    }
    out.flush()?;
    eprintln!("{frames} frames, {total} detections, {elapsed:.2} s, fps {fps}");
    Ok(())
}
