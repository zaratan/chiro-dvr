use std::fs;
use std::path::{Path, PathBuf};

use draw::text::Typeface;
use media::encode::{self, Codec};
use types::track::Track;

use super::Rendering;
use crate::clips::Error;
use crate::config::RenderConfig;
use crate::encoders::Encoders;
use crate::overlay::Overlay;
use crate::schedule::{Alone, Hold};
use crate::sequential;
use crate::source::Source;
use crate::testing::{self, Breaking, Fake, track};
use crate::windows::{Group, clip_windows, groups};

const BOTH: [Codec; 2] = [Codec::VideoToolbox, Codec::OpenH264];

#[derive(Debug)]
struct Stopped;

impl Hold for Stopped {
    fn stopped(&self) -> bool {
        true
    }

    fn keep(&self, _sessions: usize) {}
}

#[derive(Debug)]
struct Setting {
    source: Source,
    tracks: Vec<Track>,
    paths: Vec<PathBuf>,
    config: RenderConfig,
}

impl Setting {
    fn two_overlapping_clips(folder: &Path) -> Self {
        let path = testing::binoculars_of(folder, 320, 240);
        let read = sequential::read(&path);
        let points = |row: f64| -> Vec<(usize, f64, f64)> { (3..=7).map(|frame| (frame, 40.0, row)).collect() };
        Self {
            source: Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap(),
            tracks: vec![track(1, &points(40.0)), track(2, &points(120.0))],
            paths: vec![folder.join("first.mp4"), folder.join("second.mp4")],
            config: RenderConfig::new(12, 1.0, 0.1, 300).unwrap(),
        }
    }

    fn group(&self) -> Group {
        let windows = clip_windows(&self.tracks, self.source.frame_count(), self.config, self.source.rate());
        let mut all = groups(&windows);
        assert_eq!(all.len(), 1);
        all.remove(0)
    }

    fn overlay(&self) -> Overlay {
        let typeface = Typeface::roboto().unwrap();
        Overlay::new(
            &self.tracks,
            self.source.frame_count(),
            self.config,
            self.source.rate(),
            &typeface,
        )
    }

    fn paths(&self) -> Vec<Option<&Path>> {
        self.paths.iter().map(|path| Some(path.as_path())).collect()
    }
}

fn names_in(folder: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn a_videotoolbox_failure_mid_clip_is_handed_back_to_be_written_again_and_sets_videotoolbox_aside() {
    let folder = tempfile::tempdir().unwrap();
    let setting = Setting::two_overlapping_clips(folder.path());
    let group = setting.group();
    let overlay = setting.overlay();
    let fake = Fake::breaking(vec![("first.mp4", Codec::VideoToolbox, Breaking::PushAt(2))]);
    let encoders = Encoders::new(&fake, &BOTH, setting.source.rate());
    let paths = setting.paths();
    let rendering = Rendering {
        source: &setting.source,
        overlay: &overlay,
        encoders: &encoders,
        paths: &paths,
    };
    let written = rendering.render(&group, &mut None, &Alone).unwrap();
    assert_eq!(written.redo.len(), 1);
    assert_eq!(written.redo[0].path, setting.paths[0]);
    assert_eq!(written.redo[0].window.track, 0);
    assert!(matches!(written.redo[0].cause, encode::Error::FrameLayout));
    assert_eq!(encoders.hardware_sessions(&group), 0);
    assert_eq!(names_in(folder.path()), ["binoculars.mp4", "second.mp4"]);
}

#[test]
fn a_stop_raised_before_the_first_frame_ends_the_group_without_any_file() {
    let folder = tempfile::tempdir().unwrap();
    let setting = Setting::two_overlapping_clips(folder.path());
    let group = setting.group();
    let overlay = setting.overlay();
    let fake = Fake::default();
    let encoders = Encoders::new(&fake, &BOTH, setting.source.rate());
    let paths = setting.paths();
    let rendering = Rendering {
        source: &setting.source,
        overlay: &overlay,
        encoders: &encoders,
        paths: &paths,
    };
    let stopped = rendering.render(&group, &mut None, &Stopped);
    assert!(matches!(stopped, Err(Error::Stopped)));
    assert_eq!(fake.opened().len(), 0);
    assert_eq!(names_in(folder.path()), ["binoculars.mp4"]);
}
