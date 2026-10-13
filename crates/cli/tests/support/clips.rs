use std::fs;
use std::path::Path;

use media::decode::{Decoder, Threads};
use media::frame::{ColorRange, YuvFrame};

#[cfg(test)]
pub(crate) fn lengths(folder: &Path) -> Vec<(String, usize)> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let count = frame_count(&folder.join(&name));
            (name, count)
        })
        .collect()
}

#[cfg(test)]
pub(crate) fn frame_count(path: &Path) -> usize {
    let mut decoder = Decoder::open(path, Threads::Automatic).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut count = 0;
    while decoder.next(&mut frame).unwrap().is_some() {
        count += 1;
    }
    count
}
