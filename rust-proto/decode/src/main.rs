use std::env;
use std::fs::File;
use std::io::{BufWriter, Write};

use ffmpeg_next as ffmpeg;
use ffmpeg::format::Pixel;
use ffmpeg::software::scaling::{Context, Flags};
use ffmpeg::util::frame::video::Video;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let (input, output, limit) = (&args[1], &args[2], args[3].parse::<usize>()?);
    ffmpeg::init()?;
    ffmpeg::log::set_level(ffmpeg::log::Level::Quiet);
    let mut ictx = ffmpeg::format::input(input)?;
    let stream = ictx.streams().best(ffmpeg::media::Type::Video).ok_or("no video stream")?;
    let index = stream.index();
    let mut context = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
    context.set_threading(ffmpeg::threading::Config { kind: ffmpeg::threading::Type::Frame, count: 0 });
    let mut decoder = context.decoder().video()?;
    let mut scaler = Context::get(
        decoder.format(),
        decoder.width(),
        decoder.height(),
        Pixel::BGR24,
        decoder.width(),
        decoder.height(),
        Flags::BICUBIC,
    )?;
    let mut out = BufWriter::new(File::create(output)?);
    let (mut written, width, height) = (0usize, decoder.width() as usize, decoder.height() as usize);
    let mut emit = |decoder: &mut ffmpeg::decoder::Video, written: &mut usize| -> Result<(), Box<dyn std::error::Error>> {
        let mut decoded = Video::empty();
        while *written < limit && decoder.receive_frame(&mut decoded).is_ok() {
            let mut bgr = Video::empty();
            scaler.run(&decoded, &mut bgr)?;
            let stride = bgr.stride(0);
            let data = bgr.data(0);
            for row in 0..height {
                out.write_all(&data[row * stride..row * stride + width * 3])?;
            }
            *written += 1;
        }
        Ok(())
    };
    for (s, packet) in ictx.packets() {
        if written >= limit {
            break;
        }
        if s.index() == index {
            decoder.send_packet(&packet)?;
            emit(&mut decoder, &mut written)?;
        }
    }
    decoder.send_eof()?;
    emit(&mut decoder, &mut written)?;
    out.flush()?;
    eprintln!("{written} frames {width}x{height}");
    Ok(())
}
