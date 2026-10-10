//! framesrc VIDEO [SEEK:]PTS_FIRST:PTS_LAST [...]: for each inclusive pts range, writes every decoded frame
//! whose pts falls in it, as its pts (8 bytes, little endian) followed by its BGR24 pixels, decoding
//! from the key frame before the range (or before SEEK, to replay the decoder's concealment of a
//! damaged span the way a reading from the start does). Frames are chosen by pts, never by sample count: a video
//! that loses frames keeps the numbering of the detection.
//! framesrc VIDEO --pts: prints the pts of every decoded frame, one per line.

use std::collections::VecDeque;
use std::env;
use std::io::{BufWriter, Write};
use std::sync::mpsc::{SyncSender, sync_channel};

use ffmpeg::ffi;
use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::util::frame::video::Video;
use ffmpeg_next as ffmpeg;

const DEPTH: usize = 16;
const NO_PTS: i64 = i64::MIN;

fn open(input: &str, threads: usize) -> Result<(ffmpeg::format::context::Input, usize, ffmpeg::decoder::Video), ffmpeg::Error> {
    let ictx = ffmpeg::format::input(&input)?;
    let index = ictx.streams().best(ffmpeg::media::Type::Video).ok_or(ffmpeg::Error::StreamNotFound)?.index();
    let mut context = ffmpeg::codec::context::Context::from_parameters(ictx.stream(index).unwrap().parameters())?;
    context.set_threading(ffmpeg::threading::Config { kind: ffmpeg::threading::Type::Frame, count: threads });
    let decoder = context.decoder().video()?;
    Ok((ictx, index, decoder))
}

/// Decodes from `seek` (or the start) and hands each frame to `visit` until it returns false.
fn decode(
    ictx: &mut ffmpeg::format::context::Input,
    index: usize,
    decoder: &mut ffmpeg::decoder::Video,
    seek: Option<i64>,
    mut visit: impl FnMut(&Video) -> Result<bool, ffmpeg::Error>,
) -> Result<(), ffmpeg::Error> {
    if let Some(ts) = seek {
        let rc = unsafe { ffi::av_seek_frame(ictx.as_mut_ptr(), index as i32, ts, ffi::AVSEEK_FLAG_BACKWARD) };
        if rc < 0 {
            return Err(ffmpeg::Error::from(rc));
        }
    }
    let mut frame = Video::empty();
    let mut eof = false;
    let mut packets = ictx.packets();
    loop {
        while decoder.receive_frame(&mut frame).is_ok() {
            if !visit(&frame)? {
                return Ok(());
            }
        }
        if eof {
            return Ok(());
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
}

fn serve_range(input: &str, seek: i64, first: i64, last: i64, tx: SyncSender<Vec<u8>>) -> Result<(), ffmpeg::Error> {
    let (mut ictx, index, mut decoder) = open(input, 3)?;
    let (w, h) = (decoder.width(), decoder.height());
    let mut scaler = Context::get(decoder.format(), w, h, Pixel::BGR24, w, h, Flags::BICUBIC)?;
    let row = w as usize * 3;
    decode(&mut ictx, index, &mut decoder, Some(seek), |frame| {
        let Some(pts) = frame.pts() else { return Ok(true) };
        if pts < first {
            return Ok(true);
        }
        if pts > last {
            return Ok(false);
        }
        let mut bgr = Video::empty();
        scaler.run(frame, &mut bgr)?;
        let stride = bgr.stride(0);
        let mut packed = Vec::with_capacity(8 + row * h as usize);
        packed.extend_from_slice(&pts.to_le_bytes());
        for y in 0..h as usize {
            packed.extend_from_slice(&bgr.data(0)[y * stride..y * stride + row]);
        }
        Ok(tx.send(packed).is_ok() && pts < last)
    })
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let input = args[1].as_str();
    ffmpeg::init()?;
    ffmpeg::log::set_level(ffmpeg::log::Level::Quiet);
    let mut out = BufWriter::with_capacity(1 << 22, std::io::stdout().lock());
    if args.get(2).map(String::as_str) == Some("--pts") {
        let (mut ictx, index, mut decoder) = open(input, 0)?;
        decode(&mut ictx, index, &mut decoder, None, |frame| {
            writeln!(out, "{}", frame.pts().unwrap_or(NO_PTS)).map_err(|_| ffmpeg::Error::Exit)?;
            Ok(true)
        })?;
        out.flush()?;
        return Ok(());
    }
    let ranges: Vec<(i64, i64, i64)> = args[2..]
        .iter()
        .map(|r| {
            let parts: Vec<i64> = r.split(':').map(|v| v.parse().expect("pts")).collect();
            match parts[..] {
                [first, last] => (first, first, last),
                [seek, first, last] => (seek.min(first), first, last),
                _ => panic!("expected [SEEK:]PTS_FIRST:PTS_LAST"),
            }
        })
        .collect();
    let ahead = env::var("FRAMESRC_AHEAD").ok().and_then(|v| v.parse().ok()).unwrap_or(4usize);
    std::thread::scope(|scope| -> Result<(), Box<dyn std::error::Error>> {
        let spawn = |k: usize| {
            let (tx, rx) = sync_channel::<Vec<u8>>(DEPTH);
            let (seek, first, last) = ranges[k];
            let handle = scope.spawn(move || serve_range(input, seek, first, last, tx));
            (rx, handle)
        };
        let mut pending: VecDeque<_> = (0..ranges.len().min(ahead)).map(spawn).collect();
        let mut launched = pending.len();
        while let Some((rx, handle)) = pending.pop_front() {
            for frame in rx {
                out.write_all(&frame)?;
            }
            handle.join().expect("range thread")?;
            if launched < ranges.len() {
                pending.push_back(spawn(launched));
                launched += 1;
            }
        }
        Ok(())
    })?;
    out.flush()?;
    Ok(())
}
