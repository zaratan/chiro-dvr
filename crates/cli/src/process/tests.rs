use std::cell::RefCell;
use std::fs;
use std::io;
use std::num::{NonZeroU32, NonZeroUsize};
use std::path::{Path, PathBuf};

use engine::config::DetectConfig;
use engine::detection::detect;
use files::names::{Names, partial_path};
use files::previous::Kept;
use files::table::{Column, Headings};
use media::decode::{Decoder, Threads};
use media::encode::fallback::{self, Fallback};
use media::encode::{self, Codec};
use media::frame::{ColorRange, YuvFrame};
use render::clips::Rendered;
use render::config::RenderConfig;
use render::windows::clip_windows;
use summary::header::{Band, HeaderLine, Labels};
use tracking::exclusion::Exclusion;
use types::detection::{Detection, FrameDetections};
use types::frame::{FrameIndex, Pts};
use types::language::Language;
use types::rate::FrameRate;
use types::rational::Rational;
use types::timeline::Timeline;
use types::track::Track;

use super::{Clips, Error, Extent, Versions, Wording, extent, process, render_clips};
use crate::flight;
use crate::jobs::Job;
use crate::options::Options;
use crate::progress::{Progress, Silent, Step};

fn still(id: usize) -> Track {
    let points = (0..3)
        .map(|frame| Detection {
            frame: FrameIndex::new(frame),
            x: 100.0,
            y: 100.0,
            left: 95.0,
            top: 95.0,
            width: 10.0,
            height: 10.0,
            area: 100.0,
            amplitude: 40.0,
        })
        .collect();
    Track::new(id, points).unwrap()
}

const MEASURED: Extent = Extent {
    length_s: 4.0,
    not_analysed_s: 0.5,
};

fn rendered_into_a_fresh_folder(
    folder: &Path,
    max_tracks: usize,
    trial: Result<Vec<Codec>, Error>,
) -> (Result<Rendered, Error>, PathBuf) {
    let video = folder.join("flight.mp4");
    flight::write_flight(&video);
    let detected = detect(&video, Threads::Automatic, &DetectConfig::default()).unwrap();
    let reading = detected.reading();
    let pts = reading.frames().iter().map(|frame| frame.pts()).collect();
    let timeline = Timeline::new(pts, reading.video().time_base(), reading.video().frame_rate()).unwrap();
    let job = Job {
        video,
        dest: folder.join("out"),
    };
    let names = Names::new(Language::English);
    let clips = Clips {
        job: &job,
        names,
        detected: &detected,
        timeline: &timeline,
        options: &Options {
            render: RenderConfig::new(12, 1.0, 1.5, max_tracks).unwrap(),
            annotated: true,
            ..Options::default()
        },
        extent: MEASURED,
        kept: &[Kept {
            path: PathBuf::from("out/flight_tracks.csv"),
            ours: true,
        }],
    };
    let rendered = render_clips(&clips, &[still(1), still(2)], trial);
    (rendered, names.clips_dir(&job.dest))
}

#[test]
fn above_the_track_limit_no_clips_folder_is_created_and_nothing_is_written() {
    let folder = tempfile::tempdir().unwrap();
    let (rendered, clips_dir) = rendered_into_a_fresh_folder(folder.path(), 1, Ok(vec![Codec::OpenH264]));
    assert!(matches!(
        rendered,
        Err(Error::TooManyTracks {
            count: 2,
            limit: 1,
            annotated: true,
            extent: MEASURED,
            ref kept,
        }) if kept.len() == 1
    ));
    assert!(!clips_dir.exists());
}

#[test]
fn without_a_working_encoder_no_clips_folder_is_created_before_the_failure_is_returned() {
    let folder = tempfile::tempdir().unwrap();
    let failed = Fallback::new(Vec::new()).map(|_| Vec::new()).map_err(Error::Encoders);
    let (rendered, clips_dir) = rendered_into_a_fresh_folder(folder.path(), 300, failed);
    assert!(matches!(rendered, Err(Error::Encoders(fallback::Error::Empty))));
    assert!(!clips_dir.exists());
}

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

impl Headings for Plain {
    fn heading(&self, column: Column, _language: Language) -> String {
        format!("{column:?}")
    }
}

impl Wording for Plain {
    fn language(&self) -> Language {
        Language::English
    }
}

const VERSIONS: Versions<'static> = Versions {
    program: "test",
    ffmpeg: "test",
};
const LONG_FLIGHT: usize = 80;
const KEY_INTERVAL: u32 = 8;
const LOST_PART: usize = 4;
const PARTS: usize = 10;
const KEPT_FRAMES: usize = 72;
const EDGE_TOLERANCE: f64 = 2.0;
const SEARCH: i32 = 10;
const EDGE_SEARCH: i32 = 40;
const INSIDE: i32 = 8;
const NEUTRAL: u8 = 12;
const BRIGHT: u8 = 85;
const RED: u8 = 160;
const DARK: u8 = 120;

fn clip_frames(path: &Path) -> Vec<YuvFrame> {
    let mut decoder = Decoder::open(path, Threads::Automatic).unwrap();
    let mut frames = Vec::new();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    while decoder.next(&mut frame).unwrap().is_some() {
        frames.push(frame.clone());
    }
    frames
}

fn luma(frame: &YuvFrame, x: i32, y: i32) -> u8 {
    frame.y()[y as usize * frame.width() as usize + x as usize]
}

fn chroma(frame: &YuvFrame, x: i32, y: i32) -> (u8, u8) {
    let index = (y as usize / 2) * (frame.width() as usize / 2) + x as usize / 2;
    (frame.u()[index], frame.v()[index])
}

fn red(frame: &YuvFrame, x: i32, y: i32) -> bool {
    chroma(frame, x, y).1 > RED && luma(frame, x, y) < DARK
}

fn bright_center(frame: &YuvFrame, around: (i32, i32)) -> (f64, f64) {
    let mut bright = Vec::new();
    for y in around.1 - SEARCH..=around.1 + SEARCH {
        for x in around.0 - SEARCH..=around.0 + SEARCH {
            let (u, v) = chroma(frame, x, y);
            if u.abs_diff(128) < NEUTRAL && v.abs_diff(128) < NEUTRAL && luma(frame, x, y) > BRIGHT {
                bright.push((f64::from(x) + 0.5, f64::from(y) + 0.5));
            }
        }
    }
    assert!(!bright.is_empty(), "no bright square around {around:?}");
    let count = bright.len() as f64;
    let (x, y) = bright
        .iter()
        .fold((0.0, 0.0), |(x, y), point| (x + point.0, y + point.1));
    (x / count, y / count)
}

fn first_red(frame: &YuvFrame, from: (i32, i32), (step_x, step_y): (i32, i32)) -> f64 {
    let found = (0..EDGE_SEARCH)
        .map(|distance| (from.0 + distance * step_x, from.1 + distance * step_y))
        .find(|&(x, y)| red(frame, x, y));
    let (x, y) = found.unwrap_or_else(|| panic!("no red edge within {EDGE_SEARCH} px of {from:?}"));
    f64::from(if step_x == 0 { y } else { x })
}

#[test]
fn a_box_follows_its_target_on_every_detected_frame_after_frames_lost_in_the_middle() {
    let folder = tempfile::tempdir().unwrap();
    let video = folder.path().join("flight.mp4");
    flight::write_flight_of(&video, LONG_FLIGHT, NonZeroU32::new(KEY_INTERVAL));
    media::payload::wipe(&video, LOST_PART..LOST_PART + 1, PARTS).unwrap();
    let job = Job {
        video: video.clone(),
        dest: folder.path().join("out"),
    };
    let processed = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &Options::default()).unwrap();
    let kept = clip_frames(&video).len();
    assert_eq!((kept, processed.tracks.len()), (KEPT_FRAMES, 2));
    let rate = FrameRate::new(Rational::new(10, 1).unwrap()).unwrap();
    let config = RenderConfig::default();
    let pad = f64::from(config.box_pad());
    let windows = clip_windows(&processed.tracks, kept, config, rate);
    let clips_dir = Names::new(Language::English).clips_dir(&job.dest);
    let mut clips: Vec<PathBuf> = fs::read_dir(&clips_dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    clips.sort();
    for (track, (window, clip)) in processed.tracks.iter().zip(windows.iter().zip(&clips)) {
        let frames = clip_frames(clip);
        for point in track.points() {
            let frame = &frames[point.frame.get() - window.span.first().get()];
            let (x, y) = (point.x as i32, point.y as i32);
            let read = [
                first_red(frame, (x, y + INSIDE), (-1, 0)),
                first_red(frame, (x + INSIDE, y - INSIDE), (0, -1)),
                first_red(frame, (x, y + INSIDE), (1, 0)),
                first_red(frame, (x, y + INSIDE), (0, 1)),
            ];
            let expected = [
                point.left - pad,
                point.top - pad,
                point.left + point.width + pad,
                point.top + point.height + pad,
            ];
            let near = read
                .iter()
                .zip(expected)
                .all(|(read, expected)| (read - expected).abs() <= EDGE_TOLERANCE);
            assert!(near, "{:?}: box {read:?}, expected {expected:?}", point.frame);
            let (bright_x, bright_y) = bright_center(frame, (x, y));
            let inside = (point.left..=point.left + point.width).contains(&bright_x)
                && (point.top..=point.top + point.height).contains(&bright_y);
            assert!(inside, "{:?}: square at ({bright_x}, {bright_y})", point.frame);
        }
    }
}

fn pts_of(path: &Path) -> Vec<Option<i64>> {
    let mut decoder = Decoder::open(path, Threads::Exactly(NonZeroUsize::MIN)).unwrap();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut pts = Vec::new();
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        pts.push(info.pts().map(Pts::get));
    }
    pts
}

#[test]
fn the_annotated_video_of_a_flight_that_loses_frames_keeps_the_frames_and_pts_of_the_source() {
    let folder = tempfile::tempdir().unwrap();
    let video = folder.path().join("flight.mp4");
    flight::write_flight_of(&video, LONG_FLIGHT, NonZeroU32::new(KEY_INTERVAL));
    media::payload::wipe(&video, LOST_PART..LOST_PART + 1, PARTS).unwrap();
    let job = Job {
        video: video.clone(),
        dest: folder.path().join("out"),
    };
    let options = Options {
        render: RenderConfig::default(),
        annotated: true,
        ..Options::default()
    };
    process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &options).unwrap();
    let annotated = Names::new(Language::English).annotated_video(&job.dest, video.file_stem().unwrap());
    let source = pts_of(&video);
    assert_eq!(source.len(), KEPT_FRAMES);
    assert_eq!(pts_of(&annotated), source);
}

const OUR_HEADER: &str =
    "id,start,end,start_s,duration_s,hits,chord_px,path_px,speed_px_s,max_area_px,peak_amplitude,filled_frames\r\n";
const ANNOTATED_HEADER: &str = "id,start,end,verdict,start_s,duration_s,hits,chord_px,path_px,speed_px_s,max_area_px,peak_amplitude,filled_frames\r\n";

fn flight_job(folder: &Path) -> Job {
    let video = folder.join("flight.mp4");
    flight::write_flight(&video);
    Job {
        video,
        dest: folder.join("out"),
    }
}

fn left_in(dest: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dest)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn the_outputs_of_an_earlier_run_in_any_language_are_replaced_by_those_of_this_run() {
    let folder = tempfile::tempdir().unwrap();
    let job = flight_job(folder.path());
    for clips in ["extraits", "split"] {
        fs::create_dir_all(job.dest.join(clips)).unwrap();
        fs::write(job.dest.join(clips).join("piste_07_1m00s.mp4"), "").unwrap();
    }
    for earlier in [
        "flight_pistes.csv",
        "flight.tracks.csv",
        "flight_resume_2_sur_3.png",
        "flight_summary_000m-010m.png",
        "flight_annotee.mp4",
        ".flight_annotated.mp4.partial",
        "flight_boxes.mp4",
    ] {
        fs::write(job.dest.join(earlier), OUR_HEADER).unwrap();
    }
    let french = Names::new(Language::French).annotated_video(&job.dest, job.video.file_stem().unwrap());
    fs::write(encode::partial_path(&french).unwrap(), "").unwrap();
    let processed = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &Options::default()).unwrap();
    assert_eq!(
        left_in(&job.dest),
        ["clips", "flight_summary.png", "flight_tracks.csv", "params.json"]
    );
    assert_eq!(processed.csv, Some(job.dest.join("flight_tracks.csv")));
    assert_eq!(processed.kept, []);
}

#[test]
fn a_table_annotated_in_the_language_of_the_run_is_kept_as_it_is_and_no_table_is_written_over_it() {
    let folder = tempfile::tempdir().unwrap();
    let job = flight_job(folder.path());
    let table = job.dest.join("flight_tracks.csv");
    fs::create_dir_all(&job.dest).unwrap();
    fs::write(&table, ANNOTATED_HEADER).unwrap();
    let processed = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &Options::default()).unwrap();
    assert_eq!(processed.csv, None);
    assert_eq!(
        processed.kept,
        [Kept {
            path: table.clone(),
            ours: true,
        }]
    );
    assert_eq!(fs::read_to_string(&table).unwrap(), ANNOTATED_HEADER);
    assert!(job.dest.join("flight_summary.png").is_file());
}

#[test]
fn a_table_annotated_in_the_other_language_is_kept_beside_the_new_table() {
    let folder = tempfile::tempdir().unwrap();
    let job = flight_job(folder.path());
    let table = job.dest.join("flight_pistes.csv");
    fs::create_dir_all(&job.dest).unwrap();
    fs::write(&table, ANNOTATED_HEADER).unwrap();
    let processed = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &Options::default()).unwrap();
    assert_eq!(processed.csv, Some(job.dest.join("flight_tracks.csv")));
    assert_eq!(
        processed.kept,
        [Kept {
            path: table.clone(),
            ours: false,
        }]
    );
    assert_eq!(fs::read_to_string(&table).unwrap(), ANNOTATED_HEADER);
    assert!(job.dest.join("flight_tracks.csv").is_file());
}

#[test]
fn a_table_annotated_under_the_name_of_the_video_in_another_case_is_never_written_over() {
    let folder = tempfile::tempdir().unwrap();
    let job = flight_job(folder.path());
    let other_case = job.dest.join("FLIGHT_tracks.csv");
    fs::create_dir_all(&job.dest).unwrap();
    fs::write(&other_case, ANNOTATED_HEADER).unwrap();
    let ignores_case = job.dest.join("flight_tracks.csv").exists();
    let processed = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &Options::default()).unwrap();
    assert_eq!(fs::read_to_string(&other_case).unwrap(), ANNOTATED_HEADER);
    assert_eq!(processed.csv.is_none(), ignores_case);
    assert_eq!(processed.kept.len(), usize::from(ignores_case));
}

#[test]
fn a_detection_that_fails_leaves_the_results_of_an_earlier_run() {
    let folder = tempfile::tempdir().unwrap();
    let job = Job {
        video: folder.path().join("flight.mp4"),
        dest: folder.path().join("out"),
    };
    fs::write(&job.video, "not a video").unwrap();
    fs::create_dir_all(job.dest.join("clips")).unwrap();
    fs::write(job.dest.join("flight_tracks.csv"), OUR_HEADER).unwrap();
    let failed = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &Options::default());
    assert!(matches!(failed, Err(Error::Detection(_))));
    assert_eq!(left_in(&job.dest), ["clips", "flight_tracks.csv"]);
}

#[test]
fn an_encoded_video_hides_its_partial_file_under_the_name_the_cleanup_removes() {
    let video = Path::new("out/flight/flight_annotee.mp4");
    assert_eq!(encode::partial_path(video), Some(partial_path(video)));
}

#[derive(Debug, Default)]
struct Recorded(RefCell<Vec<Step>>);

impl Progress for Recorded {
    fn step(&self, step: Step) -> io::Result<()> {
        self.0.borrow_mut().push(step);
        Ok(())
    }
}

#[test]
fn a_video_goes_through_detection_tracking_summary_and_clips_in_this_order() {
    let folder = tempfile::tempdir().unwrap();
    let job = flight_job(folder.path());
    let recorded = Recorded::default();
    process(
        &job,
        &Plain,
        &recorded,
        Threads::Automatic,
        VERSIONS,
        &Options::default(),
    )
    .unwrap();
    assert_eq!(*recorded.0.borrow(), Step::ALL);
}

#[test]
fn a_detection_that_fails_has_reported_only_its_first_step() {
    let folder = tempfile::tempdir().unwrap();
    let job = Job {
        video: folder.path().join("flight.mp4"),
        dest: folder.path().join("out"),
    };
    fs::write(&job.video, "not a video").unwrap();
    let recorded = Recorded::default();
    let failed = process(
        &job,
        &Plain,
        &recorded,
        Threads::Automatic,
        VERSIONS,
        &Options::default(),
    );
    assert!(matches!(failed, Err(Error::Detection(_))));
    assert_eq!(*recorded.0.borrow(), [Step::Detection]);
}

#[derive(Debug)]
struct Failing;

impl Progress for Failing {
    fn step(&self, _step: Step) -> io::Result<()> {
        Err(io::Error::from(io::ErrorKind::BrokenPipe))
    }
}

#[test]
fn a_step_that_cannot_be_shown_fails_the_video_before_its_detection() {
    let folder = tempfile::tempdir().unwrap();
    let job = flight_job(folder.path());
    let failed = process(
        &job,
        &Plain,
        &Failing,
        Threads::Automatic,
        VERSIONS,
        &Options::default(),
    );
    assert!(matches!(failed, Err(Error::Progress(_))));
    assert!(!job.dest.exists());
}

fn timeline_from(first_pts: i64, frames: usize) -> Timeline {
    let pts = (0..frames)
        .map(|index| Some(Pts::new(first_pts + 100 * index as i64)))
        .collect();
    let rate = FrameRate::new(Rational::new(10, 1).unwrap()).unwrap();
    Timeline::new(pts, Rational::new(1, 1000).unwrap(), rate).unwrap()
}

fn excluding(frames: usize, ignored_frame_count: usize) -> Exclusion {
    Exclusion {
        detections: FrameDetections::new(vec![Vec::new(); frames]).unwrap(),
        damaged: Vec::new(),
        unstable: Vec::new(),
        reported_frames: 0,
        ignored_frame_count,
        damaged_frame_count: 0,
    }
}

#[test]
fn the_length_of_a_video_runs_from_its_first_frame_to_the_end_of_its_last_one_whatever_its_first_pts() {
    for first_pts in [0, 2500] {
        let measured = extent(&timeline_from(first_pts, 10), &excluding(10, 0));
        assert!((measured.length_s - 1.0).abs() < 1e-9, "{first_pts}: {measured:?}");
        assert!(measured.not_analysed_s.abs() < 1e-9, "{measured:?}");
    }
}

#[test]
fn the_time_not_analysed_converts_the_frames_left_out_into_seconds() {
    let measured = extent(&timeline_from(0, 10), &excluding(10, 3));
    assert!((measured.not_analysed_s - 0.3).abs() < 1e-9, "{measured:?}");
}

#[test]
fn a_processed_video_reports_its_length() {
    let folder = tempfile::tempdir().unwrap();
    let job = flight_job(folder.path());
    let processed = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &Options::default()).unwrap();
    let frames = clip_frames(&job.video).len();
    let expected = frames as f64 / 10.0;
    assert!(
        (processed.extent.length_s - expected).abs() < 1e-6,
        "{:?} for {frames} frames",
        processed.extent
    );
}

#[test]
fn a_video_with_too_many_tracks_brings_back_its_measures_and_the_table_left_as_it_is() {
    let folder = tempfile::tempdir().unwrap();
    let video = folder.path().join("flight.mp4");
    flight::write_flight_of(&video, LONG_FLIGHT, NonZeroU32::new(KEY_INTERVAL));
    media::payload::wipe(&video, LOST_PART..LOST_PART + 1, PARTS).unwrap();
    let job = Job {
        video,
        dest: folder.path().join("out"),
    };
    fs::create_dir_all(&job.dest).unwrap();
    fs::write(job.dest.join("flight_tracks.csv"), ANNOTATED_HEADER).unwrap();
    let options = Options {
        render: RenderConfig::new(12, 1.0, 1.5, 1).unwrap(),
        ..Options::default()
    };
    let failed = process(&job, &Plain, &Silent, Threads::Automatic, VERSIONS, &options);
    let Err(Error::TooManyTracks {
        count, extent, kept, ..
    }) = failed
    else {
        panic!("{failed:?}");
    };
    assert_eq!(count, 2);
    assert!(extent.length_s > 0.0 && extent.not_analysed_s > 0.0, "{extent:?}");
    assert_eq!(
        kept,
        [Kept {
            path: job.dest.join("flight_tracks.csv"),
            ours: true,
        }]
    );
}
