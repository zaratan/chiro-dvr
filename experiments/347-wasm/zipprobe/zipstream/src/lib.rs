use std::io::{self, Write};

use zip::write::{SimpleFileOptions, ZipWriter};
use zip::CompressionMethod;

pub struct Chunks<F: FnMut(&[u8])> {
    pub sent: u64,
    pub largest: usize,
    send: F,
}

impl<F: FnMut(&[u8])> Write for Chunks<F> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        (self.send)(bytes);
        self.sent += bytes.len() as u64;
        self.largest = self.largest.max(bytes.len());
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn archive<F: FnMut(&[u8])>(entries: &[(&str, u64)], chunk: usize, send: F) -> zip::result::ZipResult<(u64, usize)> {
    let sink = Chunks { sent: 0, largest: 0, send };
    let mut zip = ZipWriter::new_stream(sink);
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .large_file(true);
    let block = vec![0x5a_u8; chunk];
    for &(name, size) in entries {
        zip.start_file(name, options)?;
        let mut left = size;
        while left > 0 {
            let part = left.min(chunk as u64) as usize;
            zip.write_all(&block[..part])?;
            left -= part as u64;
        }
    }
    let sink = zip.finish()?.into_inner();
    Ok((sink.sent, sink.largest))
}

#[unsafe(no_mangle)]
pub extern "C" fn probe(size: u64) -> u64 {
    let mut total = 0_u64;
    archive(&[("video/extraits/a.mp4", size), ("video/params.json", 100)], 65_536, |bytes| total += bytes.len() as u64)
        .map_or(0, |(sent, _)| sent + total)
}
