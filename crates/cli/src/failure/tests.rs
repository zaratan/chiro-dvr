use std::fs;
use std::io;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use engine::config::{self, DetectConfig};
use engine::detection;
use engine::input;
use engine::lost::Losses;
use engine::settings::{DetectSettings, Region};
use media::decode::Threads;
use render::config::RenderConfig;
use summary::header::{Band, HeaderLine, Labels};
use summary::image_file;
use types::language::Language;

use super::{Cause, cause};
use crate::flight;
use crate::jobs::Job;
use crate::options::Options;
use crate::overview;
use crate::process::{Error, Versions, Wording, process};
use crate::progress::Silent;

const VERSIONS: Versions<'static> = Versions {
    program: "test",
    ffmpeg: "test",
};
const LONG_FLIGHT: usize = 80;
const KEY_INTERVAL: u32 = 8;
const LOST_PART: usize = 4;
const PARTS: usize = 10;

#[derive(Debug)]
struct Plain;

impl Labels for Plain {
    fn line(&self, line: &HeaderLine) -> String {
        format!("{line:?}")
    }

    fn band(&self, band: Band) -> String {
        format!("{band:?}")
    }
}

impl Wording for Plain {
    fn language(&self) -> Language {
        Language::English
    }
}

fn processed(video: PathBuf, dest: PathBuf, options: &Options) -> Cause {
    let job = Job { video, dest };
    let error = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, options).unwrap_err();
    cause(&error, options)
}

fn in_a_folder(folder: &Path, name: &str) -> (PathBuf, PathBuf) {
    (folder.join(name), folder.join("out"))
}

fn with_width(work_width: u32) -> Options {
    let settings = DetectSettings {
        work_width,
        ..DetectSettings::default()
    };
    Options {
        detect: DetectConfig::try_from(&settings).unwrap(),
        ..Options::default()
    }
}

#[test]
fn a_video_that_does_not_exist_is_missing() {
    let folder = tempfile::tempdir().unwrap();
    let (video, dest) = in_a_folder(folder.path(), "absent.mp4");
    assert_eq!(processed(video, dest, &Options::default()), Cause::Missing);
}

#[test]
fn a_text_renamed_as_a_video_is_not_a_video() {
    let folder = tempfile::tempdir().unwrap();
    let (video, dest) = in_a_folder(folder.path(), "notes.mp4");
    fs::write(&video, "not a video").unwrap();
    assert_eq!(processed(video, dest, &Options::default()), Cause::NotAVideo);
}

#[test]
fn a_video_cut_short_by_an_incomplete_copy_reads_as_not_a_video() {
    let folder = tempfile::tempdir().unwrap();
    let (video, dest) = in_a_folder(folder.path(), "flight.mp4");
    flight::write_flight(&video);
    let bytes = fs::read(&video).unwrap();
    fs::write(&video, &bytes[..bytes.len() / 2]).unwrap();
    assert_eq!(processed(video, dest, &Options::default()), Cause::NotAVideo);
}

#[cfg(unix)]
#[test]
fn a_video_the_system_refuses_to_read_is_a_read_refusal_and_not_a_write_refusal() {
    use std::os::unix::fs::PermissionsExt;

    let folder = tempfile::tempdir().unwrap();
    let (video, dest) = in_a_folder(folder.path(), "flight.mp4");
    flight::write_flight(&video);
    fs::set_permissions(&video, fs::Permissions::from_mode(0o000)).unwrap();
    let found = processed(video.clone(), dest, &Options::default());
    fs::set_permissions(&video, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(found, Cause::ReadRefused);
}

#[cfg(unix)]
#[test]
fn results_in_a_folder_without_write_permission_are_a_write_refusal() {
    use std::os::unix::fs::PermissionsExt;

    let folder = tempfile::tempdir().unwrap();
    let video = folder.path().join("flight.mp4");
    flight::write_flight(&video);
    let locked = folder.path().join("locked");
    fs::create_dir(&locked).unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o555)).unwrap();
    let found = processed(video, locked.join("flight"), &Options::default());
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(found, Cause::Forbidden);
}

#[test]
fn more_tracks_than_the_limit_keep_their_count_and_limit() {
    let folder = tempfile::tempdir().unwrap();
    let (video, dest) = in_a_folder(folder.path(), "flight.mp4");
    flight::write_flight_of(&video, LONG_FLIGHT, NonZeroU32::new(KEY_INTERVAL));
    media::payload::wipe(&video, LOST_PART..LOST_PART + 1, PARTS).unwrap();
    let options = Options {
        render: RenderConfig::new(12, 1.0, 1.5, 1).unwrap(),
        ..Options::default()
    };
    assert_eq!(
        processed(video, dest, &options),
        Cause::TooManyTracks {
            count: 2,
            limit: 1,
            annotated: false
        }
    );
}

#[test]
fn a_video_whose_stream_no_decoder_of_this_ffmpeg_reads_is_not_handled() {
    let folder = tempfile::tempdir().unwrap();
    let (video, dest) = in_a_folder(folder.path(), "flight.mp4");
    flight::write_flight(&video);
    let mut bytes = fs::read(&video).unwrap();
    let entries: Vec<usize> = bytes
        .windows(4)
        .enumerate()
        .filter(|(_, window)| *window == b"avc1")
        .map(|(at, _)| at)
        .collect();
    for at in entries {
        bytes[at..at + 4].copy_from_slice(b"mp4v");
    }
    fs::write(&video, bytes).unwrap();
    assert_eq!(processed(video, dest, &Options::default()), Cause::Unsupported);
}

#[test]
fn a_frame_count_that_does_not_add_up_is_a_damaged_video() {
    let error = Error::Detection(detection::Error::Input(input::Error::FrameCount {
        path: PathBuf::from("v.mp4"),
        announced: 30,
        losses: Losses::default(),
        decoded: 20,
    }));
    assert_eq!(cause(&error, &Options::default()), Cause::Damaged);
}

#[test]
fn a_detection_setup_that_fails_blames_the_settings_only_when_some_were_given() {
    let error = || {
        Error::Detection(detection::Error::Config {
            path: PathBuf::from("v.mp4"),
            source: config::Error::FewSamples {
                step: 4,
                frames: 8,
                samples: 2,
            },
        })
    };
    assert_eq!(cause(&error(), &Options::default()), Cause::Unsupported);
    assert_eq!(cause(&error(), &with_width(480)), Cause::Settings);
    let masked = DetectSettings {
        osd_regions: vec!["0,0,1,0.05".parse::<Region>().unwrap()],
        ..DetectSettings::default()
    };
    let options = Options {
        detect: DetectConfig::try_from(&masked).unwrap(),
        ..Options::default()
    };
    assert_eq!(cause(&error(), &options), Cause::Settings);
}

#[test]
fn a_full_disk_is_found_below_the_results_and_below_a_transparent_summary_error() {
    let full = || io::Error::from(io::ErrorKind::StorageFull);
    let table = Error::Files(files::error::Error::Write {
        path: PathBuf::from("out/v/v_pistes.csv"),
        source: full(),
    });
    let image = Error::Overview(overview::Error::Image(image_file::Error::Write {
        path: PathBuf::from("out/v/v_resume.png"),
        source: full(),
    }));
    for error in [table, image] {
        assert_eq!(cause(&error, &Options::default()), Cause::DiskFull, "{error}");
    }
}

#[test]
fn a_read_only_destination_is_a_write_refusal_but_a_broken_progress_line_is_unexpected() {
    let error = Error::Folder {
        dest: PathBuf::from("out/v"),
        source: io::Error::from(io::ErrorKind::ReadOnlyFilesystem),
    };
    assert_eq!(cause(&error, &Options::default()), Cause::Forbidden);
    let progress = Error::Progress(io::Error::from(io::ErrorKind::StorageFull));
    assert_eq!(cause(&progress, &Options::default()), Cause::Unexpected);
}

#[test]
fn a_video_whose_size_cannot_be_read_is_missing_or_refused_as_when_it_cannot_be_opened() {
    let causes = [
        io::ErrorKind::NotFound,
        io::ErrorKind::PermissionDenied,
        io::ErrorKind::Interrupted,
    ]
    .map(|kind| {
        let error = Error::Video {
            path: PathBuf::from("nuit/v.mp4"),
            source: io::Error::from(kind),
        };
        cause(&error, &Options::default())
    });
    assert_eq!(causes, [Cause::Missing, Cause::ReadRefused, Cause::Unexpected]);
}
