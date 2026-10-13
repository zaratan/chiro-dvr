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
            let mut decoder = Decoder::open(&folder.join(&name), Threads::Automatic).unwrap();
            let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
            let mut count = 0;
            while decoder.next(&mut frame).unwrap().is_some() {
                count += 1;
            }
            (name, count)
        })
        .collect()
}
