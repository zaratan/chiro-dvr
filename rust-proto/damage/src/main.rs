use std::collections::HashMap;
use std::env;
use std::ffi::{CStr, c_char, c_int, c_void};
use std::io::{BufWriter, Write, stdout};
use std::sync::{LazyLock, Mutex};
use std::time::Instant;

use ffmpeg::ffi;
use ffmpeg::util::frame::video::Video;
use ffmpeg_next as ffmpeg;

const LINE_CAPACITY: usize = 1024;
const NO_PTS: i64 = i64::MIN;

#[derive(Default)]
struct Journal {
    since_last_output: Vec<String>,
    by_pts: HashMap<i64, Vec<String>>,
    pts_of_context: HashMap<usize, i64>,
}

static JOURNAL: LazyLock<Mutex<Journal>> = LazyLock::new(|| Mutex::new(Journal::default()));

unsafe extern "C" fn record_log(context: *mut c_void, level: c_int, format: *const c_char, args: ffi::va_list) {
    if level > ffi::AV_LOG_ERROR {
        return;
    }
    let mut line = [0 as c_char; LINE_CAPACITY];
    let mut prefix: c_int = 1;
    unsafe {
        ffi::av_log_format_line2(context, level, format, args, line.as_mut_ptr(), LINE_CAPACITY as c_int, &mut prefix);
    }
    let message = unsafe { CStr::from_ptr(line.as_ptr()) }.to_string_lossy().trim_end().to_owned();
    let mut journal = JOURNAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    journal.since_last_output.push(message.clone());
    if let Some(&pts) = journal.pts_of_context.get(&(context as usize)) {
        journal.by_pts.entry(pts).or_default().push(message);
    }
}

unsafe extern "C" fn remember_pts(context: *mut ffi::AVCodecContext, frame: *mut ffi::AVFrame, flags: c_int) -> c_int {
    let pts = unsafe { (*frame).pts };
    JOURNAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .pts_of_context
        .insert(context as usize, pts);
    unsafe { ffi::avcodec_default_get_buffer2(context, frame, flags) }
}

struct Counts {
    frames: usize,
    send_errors: usize,
    receive_errors: usize,
}

fn emit(frame: &Video, counts: &mut Counts, out: &mut impl Write) -> std::io::Result<()> {
    let raw = unsafe { &*frame.as_ptr() };
    let pts = raw.pts;
    let (next, attached) = {
        let mut journal = JOURNAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let next = std::mem::take(&mut journal.since_last_output);
        let attached = journal.by_pts.remove(&pts).unwrap_or_default();
        (next, attached)
    };
    let shown = if pts == ffi::AV_NOPTS_VALUE { NO_PTS } else { pts };
    writeln!(
        out,
        "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        counts.frames,
        shown,
        u8::from(raw.flags & ffi::AV_FRAME_FLAG_KEY != 0),
        u8::from(raw.flags & ffi::AV_FRAME_FLAG_CORRUPT != 0),
        raw.decode_error_flags,
        next.len(),
        attached.len(),
        attached.first().map_or("", String::as_str),
    )?;
    counts.frames += 1;
    Ok(())
}

fn drain(decoder: &mut ffmpeg::decoder::Video, counts: &mut Counts, out: &mut impl Write) -> std::io::Result<()> {
    let mut frame = Video::empty();
    loop {
        match decoder.receive_frame(&mut frame) {
            Ok(()) => emit(&frame, counts, out)?,
            Err(ffmpeg::Error::Eof) => return Ok(()),
            Err(ffmpeg::Error::Other { errno }) if errno == ffmpeg::error::EAGAIN => return Ok(()),
            Err(_) => counts.receive_errors += 1,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let input = args.get(1).ok_or("usage: damage-check VIDEO [THREADS]")?;
    let threads = args.get(2).map_or(Ok(0), |count| count.parse::<usize>())?;
    let started = Instant::now();
    ffmpeg::init()?;
    unsafe { ffi::av_log_set_callback(Some(record_log)) };
    let mut ictx = ffmpeg::format::input(input)?;
    let stream = ictx.streams().best(ffmpeg::media::Type::Video).ok_or("no video stream")?;
    let index = stream.index();
    let mut context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
    context.set_threading(ffmpeg::threading::Config { kind: ffmpeg::threading::Type::Frame, count: threads });
    unsafe { (*context.as_mut_ptr()).get_buffer2 = Some(remember_pts) };
    let mut decoder = context.decoder().video()?;
    let mut out = BufWriter::new(stdout().lock());
    writeln!(out, "index\tpts\tkey\tcorrupt\terror_flags\tlogs_next\tlogs_ctx\tmessage")?;
    let mut counts = Counts { frames: 0, send_errors: 0, receive_errors: 0 };
    for (packet_stream, packet) in ictx.packets() {
        if packet_stream.index() != index {
            continue;
        }
        loop {
            match decoder.send_packet(&packet) {
                Err(ffmpeg::Error::Other { errno }) if errno == ffmpeg::error::EAGAIN => {
                    drain(&mut decoder, &mut counts, &mut out)?;
                }
                Err(_) => {
                    counts.send_errors += 1;
                    break;
                }
                Ok(()) => break,
            }
        }
        drain(&mut decoder, &mut counts, &mut out)?;
    }
    decoder.send_eof()?;
    drain(&mut decoder, &mut counts, &mut out)?;
    out.flush()?;
    let active = unsafe { (*decoder.as_ptr()).thread_count };
    let leftover = JOURNAL.lock().unwrap_or_else(|poisoned| poisoned.into_inner()).since_last_output.len();
    eprintln!(
        "frames {} threads {} send_errors {} receive_errors {} unattached_logs {} seconds {:.2}",
        counts.frames,
        active,
        counts.send_errors,
        counts.receive_errors,
        leftover,
        started.elapsed().as_secs_f64(),
    );
    Ok(())
}
