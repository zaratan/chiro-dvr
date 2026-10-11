use std::path::Path;

#[cfg(test)]
mod tests;

pub(crate) fn for_ffmpeg(path: &Path) -> Option<&str> {
    path.to_str().filter(|text| !text.contains('\0'))
}
