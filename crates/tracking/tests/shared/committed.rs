use reference::location::{Reference, committed, committed_settings};
use reference::recording::Recording;

const WHOLE_VIDEOS: [&str; 3] = [
    "video_092_original_3m24-4m05",
    "video_092_original",
    "video_092_original_width_1440",
];

#[cfg(test)]
pub(crate) fn recordings() -> Vec<(String, Recording)> {
    WHOLE_VIDEOS
        .into_iter()
        .map(|name| committed(Reference::V020, name))
        .chain(committed_settings(Reference::V020).unwrap())
        .map(|path| {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            (name, Recording::read(&path).unwrap())
        })
        .collect()
}
