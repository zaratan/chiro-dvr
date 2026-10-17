use std::fs;
use std::path::Path;

const APPLE_DOUBLE_PREFIX: &str = "._";

#[cfg(test)]
pub(crate) fn left_in(dest: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dest)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .filter(|name| !name.starts_with(APPLE_DOUBLE_PREFIX))
        .collect();
    names.sort();
    names
}
