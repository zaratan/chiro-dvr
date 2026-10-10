use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

#[cfg(test)]
mod tests;

const LFS_POINTER: &[u8; 42] = b"version https://git-lfs.github.com/spec/v1";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cannot read the fixture {}", path.display())]
    Unreadable {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("the fixture {} is a Git LFS pointer: run `git lfs pull`", path.display())]
    LfsPointer { path: PathBuf },
}

pub fn fixture(name: &str) -> Result<PathBuf, Error> {
    checked(fixtures().join(name))
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

fn checked(path: PathBuf) -> Result<PathBuf, Error> {
    let mut head = [0; LFS_POINTER.len()];
    match File::open(&path).and_then(|mut file| file.read_exact(&mut head)) {
        Ok(()) if head == *LFS_POINTER => Err(Error::LfsPointer { path }),
        Ok(()) => Ok(path),
        Err(error) if error.kind() == io::ErrorKind::UnexpectedEof => Ok(path),
        Err(source) => Err(Error::Unreadable { path, source }),
    }
}
