use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use draw::canvas::Canvas;
use png::{BitDepth, ColorType, Encoder};
use tempfile::TempPath;

#[cfg(test)]
mod tests;

const RGBA_CHANNELS: usize = 4;
const RGB_CHANNELS: usize = 3;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot create {}", path.display())]
    Create {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("cannot encode {}", path.display())]
    Encode {
        path: PathBuf,
        #[source]
        source: png::EncodingError,
    },
    #[error("cannot write {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

pub fn write_png(path: &Path, canvas: &Canvas) -> Result<(), Error> {
    let partial = partial_path(path);
    let create = |source| Error::Create {
        path: path.to_path_buf(),
        source,
    };
    let partial = TempPath::try_from_path(partial).map_err(create)?;
    let file = File::create(&partial).map_err(create)?;
    let encode = |source| Error::Encode {
        path: path.to_path_buf(),
        source,
    };
    let mut encoder = Encoder::new(&file, canvas.width(), canvas.height());
    encoder.set_color(ColorType::Rgb);
    encoder.set_depth(BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(encode)?;
    writer.write_image_data(&rgb(canvas)).map_err(encode)?;
    writer.finish().map_err(encode)?;
    partial.persist(path).map_err(|failure| Error::Write {
        path: path.to_path_buf(),
        source: failure.error,
    })
}

fn partial_path(path: &Path) -> PathBuf {
    let mut name = OsString::from(".");
    name.push(path.file_name().unwrap_or_default());
    name.push(".partial");
    path.with_file_name(name)
}

fn rgb(canvas: &Canvas) -> Vec<u8> {
    let (pixels, _) = canvas.rgba().as_chunks::<RGBA_CHANNELS>();
    let mut bytes = Vec::with_capacity(pixels.len() * RGB_CHANNELS);
    for pixel in pixels {
        bytes.extend_from_slice(&pixel[..RGB_CHANNELS]);
    }
    bytes
}
