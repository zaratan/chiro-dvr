use std::fs;
use std::path::{Path, PathBuf};

use draw::text::Typeface;
use media::decode::{Decoder, Threads};
use media::encode::{self, Codec};
use types::rational::Rational;
use types::track::Track;

use super::{Rendering, Scratch};
use crate::clips::Error;
use crate::config::RenderConfig;
use crate::encoders::Encoders;
use crate::overlay::Overlay;
use crate::schedule::{Alone, Hold};
use crate::sequential;
use crate::source::Source;
use crate::testing::{self, Breaking, Fake, track};
use crate::windows::{Group, clip_windows, groups};
use crate::work::{Paths, Work};

const BOTH: [Codec; 2] = [Codec::VideoToolbox, Codec::OpenH264];

#[derive(Debug)]
struct Stopped;

impl Hold for Stopped {
    fn stopped(&self) -> bool {
        true
    }
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
fn a_videotoolbox_failure_hands_the_whole_group_back_to_be_written_again_and_sets_videotoolbox_aside() {
    let folder = tempfile::tempdir().unwrap();
    let setting = Setting::two_overlapping_clips(folder.path());
    let group = setting.group();
    let overlay = setting.overlay();
    let fake = Fake::breaking(vec![("first.mp4", Codec::VideoToolbox, Breaking::PushAt(2))]);
    let opener = &fake;
    let encoders = Encoders::new(&opener, &BOTH, setting.source.rate());
    let paths = setting.paths();
    let rendering = Rendering {
        source: &setting.source,
        overlay: &overlay,
        encoders: &encoders,
        paths: Paths {
            clips: &paths,
            zooms: &[],
        },
        tracks: &setting.tracks,
        trail_frames: 3,
    };
    let written = rendering
        .render(&Work::Clips(group), &mut Scratch::default(), &Alone)
        .unwrap();
    assert!(matches!(written.failed, Some(encode::Error::FrameLayout)));
    assert_eq!(encoders.hardware_sessions(), 0);
    assert_eq!(names_in(folder.path()), ["binoculars.mp4"]);
}

#[test]
fn a_stop_raised_before_the_first_frame_ends_the_group_without_any_file() {
    let folder = tempfile::tempdir().unwrap();
    let setting = Setting::two_overlapping_clips(folder.path());
    let group = setting.group();
    let overlay = setting.overlay();
    let fake = Fake::default();
    let opener = &fake;
    let encoders = Encoders::new(&opener, &BOTH, setting.source.rate());
    let paths = setting.paths();
    let rendering = Rendering {
        source: &setting.source,
        overlay: &overlay,
        encoders: &encoders,
        paths: Paths {
            clips: &paths,
            zooms: &[],
        },
        tracks: &setting.tracks,
        trail_frames: 3,
    };
    let stopped = rendering.render(&Work::Clips(group), &mut Scratch::default(), &Stopped);
    assert!(matches!(stopped, Err(Error::Stopped)));
    assert_eq!(fake.opened().len(), 0);
    assert_eq!(names_in(folder.path()), ["binoculars.mp4"]);
}

#[test]
fn the_clips_of_a_group_come_from_one_encoder_session_that_opens_each_on_its_first_frame() {
    let folder = tempfile::tempdir().unwrap();
    let mut setting = Setting::two_overlapping_clips(folder.path());
    let later: Vec<(usize, f64, f64)> = (5..=9).map(|frame| (frame, 40.0, 120.0)).collect();
    setting.tracks[1] = track(2, &later);
    let group = setting.group();
    let overlay = setting.overlay();
    let fake = Fake::default();
    let opener = &fake;
    let encoders = Encoders::new(&opener, &BOTH, setting.source.rate());
    let paths = setting.paths();
    let rendering = Rendering {
        source: &setting.source,
        overlay: &overlay,
        encoders: &encoders,
        paths: Paths {
            clips: &paths,
            zooms: &[],
        },
        tracks: &setting.tracks,
        trail_frames: 3,
    };
    let written = rendering
        .render(&Work::Clips(group), &mut Scratch::default(), &Alone)
        .unwrap();
    assert!(written.failed.is_none());
    let expected = [("first.mp4", Codec::VideoToolbox), ("second.mp4", Codec::VideoToolbox)]
        .map(|(name, codec)| (name.to_owned(), codec));
    assert_eq!(fake.opened(), expected);
    assert_eq!(fake.encoders_created(), 1);
    let (early, late) = (sequential::read(&setting.paths[0]), sequential::read(&setting.paths[1]));
    assert_eq!((early.timeline.frame_count(), late.timeline.frame_count()), (11, 11));
    assert_eq!(late.fingerprints[0], early.fingerprints[2]);
    assert_eq!(late.keys.first().map(|key| key.get()), Some(0));
}

#[test]
fn a_zoom_is_written_by_openh264_at_a_quarter_of_the_rate_over_the_window_of_its_clip() {
    let folder = tempfile::tempdir().unwrap();
    let setting = Setting::two_overlapping_clips(folder.path());
    let window = clip_windows(
        &setting.tracks,
        setting.source.frame_count(),
        setting.config,
        setting.source.rate(),
    )[0];
    let overlay = setting.overlay();
    let fake = Fake::default();
    let opener = &fake;
    let encoders = Encoders::new(&opener, &BOTH, setting.source.rate());
    let zoom = folder.path().join("first_zoom.mp4");
    let zooms = [Some(zoom.as_path()), None];
    let rendering = Rendering {
        source: &setting.source,
        overlay: &overlay,
        encoders: &encoders,
        paths: Paths {
            clips: &[],
            zooms: &zooms,
        },
        tracks: &setting.tracks,
        trail_frames: 3,
    };
    let written = rendering
        .render(&Work::Zoom(window), &mut Scratch::default(), &Alone)
        .unwrap();
    assert!(written.failed.is_none());
    assert_eq!(fake.opened(), [("first_zoom.mp4".to_owned(), Codec::OpenH264)]);
    assert_eq!(fake.most_hardware_at_once(), 0);
    let read = sequential::read(&zoom);
    assert_eq!(read.timeline.frame_count(), window.span.frame_count());
    assert_eq!(read.timeline.rate().get(), Rational::new(2500, 333).unwrap());
    let size = Decoder::open(&zoom, Threads::Automatic).unwrap().stream().size();
    assert_eq!((size.width().get(), size.height().get()), (320, 240));
}
