#[path = "support/agreement.rs"]
mod agreement;
#[path = "support/clips.rs"]
mod clips;

use batdetect::options::Options;
use batdetect::process::process;
use media::decode::{Decoder, Threads};
use reference::fixture::fixture;
use reference::location::committed;
use reference::recording::Recording;
use types::language::Language;
use types::rate::FrameRate;
use types::rational::Rational;

const WHOLE: &str = "video_092_original";

#[test]
#[ignore = "reference"]
fn the_092_gives_the_fourteen_tracks_and_clips_of_the_reference_and_its_single_zoom() {
    let recording = Recording::read(&committed(WHOLE)).unwrap();
    let rate = FrameRate::new(recording.summary.stream.avg_frame_rate).unwrap();
    let out = tempfile::tempdir().unwrap();
    let video = fixture(&format!("{WHOLE}.mp4")).unwrap();
    let job = agreement::job(&video, &out.path().join(WHOLE));
    let processed = process(
        &job,
        Language::English,
        &agreement::DebugLabels,
        Threads::Automatic,
        agreement::VERSIONS,
        Options::default(),
    )
    .unwrap();
    let found = agreement::agreement(&processed, &recording, rate);
    assert_eq!(processed.tracks.len(), 14);
    assert_eq!((found.reference, found.paired, found.agreeing), (14, 14, 14));
    assert_eq!((found.lost, found.added), (0, 0));
    assert_eq!(found.discrepancies, []);
    let lengths = clips::lengths(&out.path().join(WHOLE).join("clips"));
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
    assert_eq!(lengths, expected);
    let zoom = out.path().join(WHOLE).join("clips").join("track_03_0m51s_zoom.mp4");
    let slowed = Decoder::open(&zoom, Threads::Automatic).unwrap().stream().frame_rate();
    assert_eq!(slowed.get(), Rational::new(22_527_500, 3_000_927).unwrap());
}
