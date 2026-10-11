use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

use flate2::Compression;
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::error::Error;

#[cfg(test)]
mod tests;

pub(crate) fn read<T: DeserializeOwned>(path: &Path) -> Result<Vec<T>, Error> {
    let file = File::open(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    if is_gzip(path) {
        parse(BufReader::new(MultiGzDecoder::new(file)), path)
    } else {
        parse(BufReader::new(file), path)
    }
}

pub(crate) fn write<T: Serialize>(path: &Path, items: impl IntoIterator<Item = T>) -> Result<(), Error> {
    let failed = |source| Error::Write {
        path: path.to_path_buf(),
        source,
    };
    let file = BufWriter::new(File::create(path).map_err(failed)?);
    if is_gzip(path) {
        let mut encoder = GzEncoder::new(file, Compression::default());
        emit(&mut encoder, items, path)?;
        encoder.finish().and_then(|mut file| file.flush()).map_err(failed)
    } else {
        let mut file = file;
        emit(&mut file, items, path)?;
        file.flush().map_err(failed)
    }
}

fn is_gzip(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "gz")
}

fn parse<T: DeserializeOwned>(reader: impl BufRead, path: &Path) -> Result<Vec<T>, Error> {
    reader
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let line = line.map_err(|source| Error::Read {
                path: path.to_path_buf(),
                source,
            })?;
            serde_json::from_str(&line).map_err(|source| Error::Json {
                path: path.to_path_buf(),
                line: index + 1,
                source,
            })
        })
        .collect()
}

fn emit<T: Serialize>(writer: &mut impl Write, items: impl IntoIterator<Item = T>, path: &Path) -> Result<(), Error> {
    for item in items {
        serde_json::to_writer(&mut *writer, &item).map_err(|source| Error::Encode {
            path: path.to_path_buf(),
            source,
        })?;
        writer.write_all(b"\n").map_err(|source| Error::Write {
            path: path.to_path_buf(),
            source,
        })?;
    }
    Ok(())
}
