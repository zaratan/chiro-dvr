use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};

use media::encode::{self, Codec};
use types::track::Track;

use super::{Clip, Error, Rendered, Video, render, render_with};
use crate::config::RenderConfig;
use crate::encoders::Ffmpeg;
use crate::sequential::{self, Sequential};
use crate::source::Source;
use crate::testing::{self, Breaking, Fake, track};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const BOTH: [Codec; 2] = [Codec::VideoToolbox, Codec::OpenH264];

fn tenth_of_a_second() -> RenderConfig {
    RenderConfig::new(12, 1.0, 0.1, 300).unwrap()
}

fn binoculars(folder: &Path) -> (PathBuf, Sequential) {
    let path = testing::binoculars_of(folder, WIDTH, HEIGHT);
    let read = sequential::read(&path);
    (path, read)
}

fn source(folder: &Path) -> (PathBuf, Source) {
    let (path, read) = binoculars(folder);
    let source = Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap();
    (path, source)
}

fn crossing(frames: RangeInclusive<usize>, row: f64) -> Track {
    let points: Vec<(usize, f64, f64)> = frames.map(|frame| (frame, 20.0 + 4.0 * frame as f64, row)).collect();
    track(1, &points)
}

fn tracks(spans: &[(RangeInclusive<usize>, usize)]) -> Vec<Track> {
    spans
        .iter()
        .flat_map(|(frames, count)| (0..*count).map(|row| crossing(frames.clone(), 30.0 + 20.0 * row as f64)))
        .enumerate()
        .map(|(index, track)| track.with_id(index + 1))
        .collect()
}

fn clips(folder: &Path, tracks: &[Track]) -> Vec<Clip> {
    (0..tracks.len())
        .map(|track| Clip {
            track,
            path: folder.join(format!("clip_{track}.mp4")),
        })
        .collect()
}

fn written_with(
    fake: &Fake,
    threads: usize,
    source: &Source,
    tracks: &[Track],
    clips: &[Clip],
    codecs: &[Codec],
) -> Result<Rendered, Error> {
    let video = Video {
        source,
        tracks,
        config: tenth_of_a_second(),
    };
    render_with(&fake, threads, &video, clips, codecs)
}

fn frame_count(path: &Path) -> usize {
    sequential::read(path).timeline.frame_count()
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
fn each_clip_holds_every_frame_of_its_window_bounded_by_the_video() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = tracks(&[(3..=7, 1), (8..=12, 1), (15..=17, 1), (40..=46, 1)]);
    let clips = clips(folder.path(), &tracks);
    let rendered = render(&source, &tracks, &clips, tenth_of_a_second(), &[Codec::OpenH264]).unwrap();
    assert_eq!((rendered.restarts, rendered.redone.len()), (0, 0));
    let counts: Vec<usize> = clips.iter().map(|clip| frame_count(&clip.path)).collect();
    assert_eq!(counts, [11, 11, 9, 11]);
}

#[test]
fn only_the_requested_clips_are_written() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = tracks(&[(3..=7, 1), (20..=24, 1)]);
    let wanted = folder.path().join("second.mp4");
    let clips = [Clip {
        track: 1,
        path: wanted.clone(),
    }];
    render(&source, &tracks, &clips, tenth_of_a_second(), &[Codec::OpenH264]).unwrap();
    assert_eq!(frame_count(&wanted), 11);
    assert_eq!(names_in(folder.path()), ["binoculars.mp4", "second.mp4"]);
}

#[test]
fn a_window_read_again_from_the_first_frame_is_counted() {
    let folder = tempfile::tempdir().unwrap();
    let (path, read) = binoculars(folder.path());
    let source = Source::new(&path, &read.timeline, &[], &read.errors).unwrap();
    let tracks = tracks(&[(3..=7, 1), (20..=24, 1)]);
    let clips = clips(folder.path(), &tracks);
    let rendered = render(&source, &tracks, &clips, tenth_of_a_second(), &[Codec::OpenH264]).unwrap();
    assert_eq!(rendered.restarts, 2);
}

#[test]
fn a_clip_for_a_track_the_video_does_not_hold_is_refused_before_any_writing() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = tracks(&[(3..=7, 1), (60..=64, 1)]);
    for track in [1, 2] {
        let clip = Clip {
            track,
            path: folder.path().join("clip.mp4"),
        };
        let error = render(&source, &tracks, &[clip], tenth_of_a_second(), &[Codec::OpenH264]).unwrap_err();
        assert!(matches!(error, Error::UnknownTrack { track: refused } if refused == track));
    }
    assert_eq!(names_in(folder.path()), ["binoculars.mp4"]);
}

#[test]
fn a_clip_that_cannot_be_written_there_stops_the_video_and_leaves_no_partial_file() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = tracks(&[(3..=7, 1), (5..=9, 1)]);
    let clips = [
        Clip {
            track: 0,
            path: folder.path().join("clip_0.mp4"),
        },
        Clip {
            track: 1,
            path: folder.path().join("missing").join("clip_1.mp4"),
        },
    ];
    let error = render(&source, &tracks, &clips, tenth_of_a_second(), &[Codec::OpenH264]).unwrap_err();
    assert!(matches!(error, Error::Destination { path: refused, .. } if refused == clips[1].path));
    assert_eq!(names_in(folder.path()), ["binoculars.mp4"]);
}

#[test]
fn groups_written_in_parallel_give_the_same_bytes_as_a_single_thread() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = tracks(&[(3..=7, 2), (8..=12, 1), (20..=26, 3), (40..=46, 1)]);
    let written: Vec<Vec<(usize, u64)>> = [1, 4]
        .iter()
        .map(|threads| {
            let out = folder.path().join(format!("threads_{threads}"));
            fs::create_dir(&out).unwrap();
            let clips = clips(&out, &tracks);
            let video = Video {
                source: &source,
                tracks: &tracks,
                config: tenth_of_a_second(),
            };
            render_with(&Ffmpeg, *threads, &video, &clips, &[Codec::OpenH264]).unwrap();
            clips
                .iter()
                .map(|clip| {
                    let bytes = fs::read(&clip.path).unwrap();
                    let mut hasher = DefaultHasher::new();
                    bytes.hash(&mut hasher);
                    (bytes.len(), hasher.finish())
                })
                .collect()
        })
        .collect();
    assert_eq!(written[0].len(), 7);
    assert_eq!(written[0], written[1]);
}

#[test]
fn each_group_writes_all_its_clips_through_a_single_videotoolbox_session() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = tracks(&[(3..=7, 9), (19..=23, 6), (35..=39, 6)]);
    let clips = clips(folder.path(), &tracks);
    let fake = Fake::default();
    written_with(&fake, 3, &source, &tracks, &clips, &BOTH).unwrap();
    assert!(fake.most_hardware_at_once() <= 3);
    assert_eq!(fake.encoders_created(), 3);
    assert!(clips.iter().all(|clip| frame_count(&clip.path) == 11));
    let opened = fake.opened();
    assert_eq!(opened.len(), 21);
    assert!(opened.iter().all(|(_, codec)| *codec == Codec::VideoToolbox));
}

#[test]
fn a_videotoolbox_failure_writes_its_whole_group_again_with_openh264_and_sets_videotoolbox_aside() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = tracks(&[(3..=7, 2), (20..=24, 1)]);
    let clips = clips(folder.path(), &tracks);
    let fake = Fake::breaking(vec![("clip_0.mp4", Codec::VideoToolbox, Breaking::PushAt(3))]);
    let rendered = written_with(&fake, 1, &source, &tracks, &clips, &BOTH).unwrap();
    assert_eq!(rendered.redone.len(), 1);
    assert_eq!(rendered.redone[0].paths, [clips[0].path.clone(), clips[1].path.clone()]);
    assert!(matches!(rendered.redone[0].cause, encode::Error::FrameLayout));
    let opened = [
        ("clip_0.mp4", Codec::VideoToolbox),
        ("clip_1.mp4", Codec::VideoToolbox),
        ("clip_2.mp4", Codec::OpenH264),
        ("clip_0.mp4", Codec::OpenH264),
        ("clip_1.mp4", Codec::OpenH264),
    ]
    .map(|(name, codec)| (name.to_owned(), codec));
    assert_eq!(fake.opened(), opened);
    assert!(clips.iter().all(|clip| frame_count(&clip.path) == 11));
    assert_eq!(
        names_in(folder.path()),
        ["binoculars.mp4", "clip_0.mp4", "clip_1.mp4", "clip_2.mp4"]
    );
}

#[test]
fn a_videotoolbox_failure_about_the_destination_is_not_written_again() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = tracks(&[(3..=7, 1)]);
    let clips = clips(folder.path(), &tracks);
    let fake = Fake::breaking(vec![("clip_0.mp4", Codec::VideoToolbox, Breaking::Persist)]);
    let error = written_with(&fake, 1, &source, &tracks, &clips, &BOTH).unwrap_err();
    assert!(matches!(error, Error::Destination { path, .. } if path == clips[0].path));
    assert_eq!(names_in(folder.path()), ["binoculars.mp4"]);
}

#[test]
fn an_openh264_failure_stops_the_video_and_leaves_the_later_groups_unstarted() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = tracks(&[(3..=7, 3), (30..=34, 1)]);
    let clips = clips(folder.path(), &tracks);
    let fake = Fake::breaking(vec![("clip_0.mp4", Codec::OpenH264, Breaking::PushAt(2))]);
    let error = written_with(&fake, 1, &source, &tracks, &clips, &[Codec::OpenH264]).unwrap_err();
    assert!(
        matches!(&error, Error::Clip { path, .. } if *path == clips[0].path),
        "{error:?}"
    );
    assert!(fake.opened().iter().all(|(name, _)| name != "clip_3.mp4"));
    assert_eq!(names_in(folder.path()), ["binoculars.mp4"]);
}

#[test]
fn each_clip_is_written_by_the_same_encoder_whatever_the_number_of_threads() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = tracks(&[(3..=7, 9), (20..=30, 3), (24..=26, 4), (40..=44, 6)]);
    let clips = clips(folder.path(), &tracks);
    let encoders_of = |threads: usize| {
        let fake = Fake::default();
        written_with(&fake, threads, &source, &tracks, &clips, &BOTH).unwrap();
        let mut opened = fake.opened();
        opened.sort_by(|a, b| a.0.cmp(&b.0));
        opened
    };
    let alone = encoders_of(1);
    for threads in [3, 5, 5] {
        assert_eq!(encoders_of(threads), alone);
    }
    assert!(clips.iter().all(|clip| frame_count(&clip.path) > 0));
}

#[test]
fn a_failure_on_the_second_clip_of_a_group_names_that_clip() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = [crossing(3..=7, 30.0), crossing(5..=12, 50.0)].map(|track| track.with_id(1));
    let clips = clips(folder.path(), &tracks);
    let fake = Fake::breaking(vec![("clip_1.mp4", Codec::VideoToolbox, Breaking::Persist)]);
    let error = written_with(&fake, 1, &source, &tracks, &clips, &BOTH).unwrap_err();
    assert!(
        matches!(&error, Error::Destination { path, .. } if *path == clips[1].path),
        "{error:?}"
    );
}

#[test]
fn a_videotoolbox_failure_after_a_clip_of_the_group_is_finished_writes_that_clip_again_too() {
    let folder = tempfile::tempdir().unwrap();
    let (_, source) = source(folder.path());
    let tracks = [crossing(3..=7, 30.0), crossing(5..=12, 50.0)].map(|track| track.with_id(1));
    let clips = clips(folder.path(), &tracks);
    let fake = Fake::breaking(vec![("clip_1.mp4", Codec::VideoToolbox, Breaking::PushAt(11))]);
    let rendered = written_with(&fake, 1, &source, &tracks, &clips, &BOTH).unwrap();
    assert_eq!(rendered.redone.len(), 1);
    assert_eq!(rendered.redone[0].paths, [clips[0].path.clone(), clips[1].path.clone()]);
    let opened = [
        ("clip_0.mp4", Codec::VideoToolbox),
        ("clip_1.mp4", Codec::VideoToolbox),
        ("clip_0.mp4", Codec::OpenH264),
        ("clip_1.mp4", Codec::OpenH264),
    ]
    .map(|(name, codec)| (name.to_owned(), codec));
    assert_eq!(fake.opened(), opened);
    assert_eq!((frame_count(&clips[0].path), frame_count(&clips[1].path)), (11, 14));
    assert_eq!(names_in(folder.path()), ["binoculars.mp4", "clip_0.mp4", "clip_1.mp4"]);
}
