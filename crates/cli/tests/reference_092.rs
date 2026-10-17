#[path = "support/agreement.rs"]
mod agreement;
#[path = "support/clips.rs"]
mod clips;
#[path = "support/panics.rs"]
mod panics;

use std::error::Error;
use std::io;
use std::iter::successors;
use std::panic::{self, AssertUnwindSafe, UnwindSafe};
use std::path::Path;
use std::slice;
use std::sync::OnceLock;
use std::thread;
use std::time::Instant;

use batdetect::options::Options;
use batdetect::process::{Processed, process_detected};
use batdetect::progress::Silent;
use compare::agreement::Agreement;
use compare::case::{Case, WHOLE, WHOLE_AT_1440};
use compare::comparison::{Comparison, Folders, compare, compared};
use compare::frames::pass_of;
use compare::report::write_report;
use compare::tracks::TrackCounts;
use engine::config::DetectConfig;
use engine::damage::Damage;
use engine::detection::{Detected, detect};
use engine::settings::DetectSettings;
use media::decode::{Decoder, Threads};
use media::frame::ColorRange;
use reference::fixture::fixture;
use reference::frames::Frames;
use reference::location::{Reference, committed};
use reference::recording::{Recording, SUMMARY_FILE};
use reference::video::VideoSummary;
use types::rate::FrameRate;
use types::rational::Rational;

use crate::panics::panicked;

struct Defaults {
    detected: Detected,
    detection_s: f64,
    command: Result<Command, String>,
}

struct Command {
    processed_in_removed_folder: Processed,
    clips: Vec<(String, usize)>,
    zoom_rate: Result<FrameRate, String>,
}

struct Shared {
    defaults: Result<Defaults, String>,
    at_1440: Result<Comparison, String>,
}

const V011_RATE: f64 = 0.98;

fn within_the_v011_rate(detections: &Agreement) -> bool {
    detections.reference_rate() >= V011_RATE && detections.candidate_rate() >= V011_RATE
}

fn shared() -> &'static Shared {
    static SHARED: OnceLock<Shared> = OnceLock::new();
    SHARED.get_or_init(both)
}

fn both() -> Shared {
    thread::scope(|scope| {
        let at_1440 = scope.spawn(|| caught(at_1440));
        let defaults = caught(defaults);
        Shared {
            defaults,
            at_1440: at_1440.join().unwrap_or_else(|payload| Err(panicked(&*payload))),
        }
    })
}

fn caught<T>(work: impl FnOnce() -> Result<T, Box<dyn Error>> + UnwindSafe) -> Result<T, String> {
    match panic::catch_unwind(work) {
        Ok(Ok(value)) => Ok(value),
        Ok(Err(error)) => Err(described(&*error)),
        Err(payload) => Err(panicked(&*payload)),
    }
}

fn described(error: &(dyn Error + 'static)) -> String {
    successors(Some(error), |&error| error.source())
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(": ")
}

fn defaults() -> Result<Defaults, Box<dyn Error>> {
    let options = Options::default();
    let video = fixture(&format!("{WHOLE}.mp4"))?;
    let start = Instant::now();
    let detected = detect(&video, Threads::Automatic, &options.detect)?;
    let detection_s = start.elapsed().as_secs_f64();
    let command = caught(AssertUnwindSafe(|| command(&video, &detected, &options)));
    Ok(Defaults {
        detected,
        detection_s,
        command,
    })
}

fn command(video: &Path, detected: &Detected, options: &Options) -> Result<Command, Box<dyn Error>> {
    let out = tempfile::tempdir()?;
    let dest = out.path().join(WHOLE);
    let job = agreement::job(video, &dest);
    let processed = process_detected(
        &job,
        detected,
        &agreement::DebugLabels,
        &Silent,
        agreement::VERSIONS,
        options,
    )?;
    let folder = dest.join("clips");
    let clips = clips::lengths(&folder);
    let zoom_rate = Decoder::open(&folder.join("track_03_0m51s_zoom.mp4"), Threads::Automatic)
        .map(|decoder| decoder.stream().frame_rate())
        .map_err(|error| described(&error));
    Ok(Command {
        processed_in_removed_folder: processed,
        clips,
        zoom_rate,
    })
}

fn at_1440() -> Result<Comparison, Box<dyn Error>> {
    Ok(compare(
        &Case::setting(committed(Reference::V011, WHOLE_AT_1440)),
        &Folders::default(),
    )?)
}

#[test]
#[ignore = "reference"]
fn the_092_gives_the_fourteen_tracks_and_clips_of_the_reference_and_its_single_zoom() {
    let command = shared().defaults.as_ref().unwrap().command.as_ref().unwrap();
    let recording = Recording::read(&committed(Reference::V011, WHOLE)).unwrap();
    let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
    let found = agreement::agreement(&command.processed_in_removed_folder, &recording, rate);
    assert_eq!(command.processed_in_removed_folder.tracks.len(), 14);
    assert_eq!((found.reference, found.paired, found.agreeing), (14, 14, 14));
    assert_eq!((found.lost, found.added), (0, 0));
    assert_eq!(found.discrepancies, []);
    let expected = [
        ("track_01_0m07s.mp4", 113),
        ("track_02_0m33s.mp4", 131),
        ("track_03_0m51s.mp4", 99),
        ("track_03_0m51s_zoom.mp4", 99),
        ("track_04_0m53s.mp4", 125),
        ("track_05_1m12s.mp4", 129),
        ("track_06_1m48s.mp4", 123),
        ("track_07_2m02s.mp4", 111),
        ("track_08_2m24s.mp4", 106),
        ("track_09_2m36s.mp4", 105),
        ("track_10_2m43s.mp4", 134),
        ("track_11_3m29s.mp4", 380),
        ("track_12_3m48s.mp4", 108),
        ("track_13_3m50s.mp4", 115),
        ("track_14_3m57s.mp4", 117),
    ]
    .map(|(name, frames)| (name.to_owned(), frames));
    assert_eq!(command.clips, expected);
    assert_eq!(
        command.zoom_rate.as_ref().map(|rate| rate.get()),
        Ok(Rational::new(22_527_500, 3_000_927).unwrap())
    );
}

#[test]
#[ignore = "reference"]
fn the_whole_092_agrees_with_the_reference_on_98_percent_of_its_detections_both_ways() {
    let defaults = shared().defaults.as_ref().unwrap();
    let recording = Recording::read(&committed(Reference::V011, WHOLE)).unwrap();
    assert_eq!(recording.summary.video.name, format!("{WHOLE}.mp4"));
    let settings = recording.summary.detect_settings::<DetectSettings>().unwrap();
    assert_eq!(DetectConfig::try_from(&settings).unwrap(), Options::default().detect);
    assert_eq!(
        defaults.detected.half_window().get(),
        recording.summary.background_half_window
    );
    let name = Case::committed(Reference::V011, WHOLE).name(&recording.summary.video.name);
    let comparison = compared(
        &name,
        &pass_of(&defaults.detected).unwrap(),
        &recording,
        defaults.detection_s,
    )
    .unwrap();

    write_report(slice::from_ref(&comparison), &mut io::stdout().lock()).unwrap();
    assert!(within_the_v011_rate(&comparison.detections));
}

#[test]
#[ignore = "reference"]
fn the_whole_092_worked_at_1440_pixels_agrees_with_its_reference_on_98_percent_of_its_detections_and_on_every_track() {
    let comparison = shared().at_1440.as_ref().unwrap();

    write_report(slice::from_ref(comparison), &mut io::stdout().lock()).unwrap();
    assert!(within_the_v011_rate(&comparison.detections));
    assert_eq!(
        comparison.tracks.counts(),
        TrackCounts {
            reference: 14,
            paired: 14,
            agreeing: 14,
            lost: 0,
            added: 0,
        }
    );
}

#[test]
#[ignore = "reference"]
fn the_whole_092_reads_its_9011_frames_with_the_timestamps_key_frames_and_lack_of_damage_of_ffprobe() {
    let reading = shared().defaults.as_ref().unwrap().detected.reading();
    let recorded = committed(Reference::V011, WHOLE);
    let summary = VideoSummary::read(&recorded.join(SUMMARY_FILE)).unwrap();
    let probed = Frames::read(&recorded.join(&summary.files.frames)).unwrap();
    assert_eq!(reading.frames().len(), 9011);
    assert_eq!(reading.video().frame_rate().get(), summary.stream.avg_frame_rate);
    assert_eq!(reading.video().time_base(), summary.stream.time_base);
    let read: Vec<_> = reading
        .frames()
        .iter()
        .map(|frame| (frame.pts(), frame.is_key()))
        .collect();
    let expected: Vec<_> = probed
        .entries()
        .iter()
        .map(|entry| (entry.pts, entry.key_frame))
        .collect();
    assert_eq!(read, expected);
    let damage = Damage::of(reading, summary.background_half_window);
    let probe = summary.ffprobe.as_ref().unwrap();
    assert_eq!(damage.error_frames(), probe.error_frames);
    assert_eq!(damage.jump_frames(), probe.gap_frames);
    assert_eq!(damage.spans(), summary.damaged_spans);
    assert_eq!(damage.spans(), []);
}

#[test]
#[ignore = "reference"]
fn the_whole_092_decodes_its_9011_frames_at_its_average_rate() {
    let reading = shared().defaults.as_ref().unwrap().detected.reading();
    assert!(reading.frames().iter().all(|frame| frame.unreadable_packets() == 0));
    assert_eq!(reading.frames().len(), 9011);
    let video = reading.video();
    assert_eq!(video.frame_rate().get(), Rational::new(90_110_000, 3_000_927).unwrap());
    assert_eq!(video.time_base(), Rational::new(1, 10_000).unwrap());
    let range = Decoder::open(video.path(), Threads::Automatic)
        .unwrap()
        .stream()
        .range();
    assert_eq!(
        (video.size().width().get(), video.size().height().get(), range),
        (1440, 1080, ColorRange::Limited)
    );
}
