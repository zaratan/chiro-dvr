#[path = "support/agreement.rs"]
mod agreement;

use batdetect::process::process;
use media::decode::Threads;
use reference::fixture::fixture;
use reference::location::committed;
use reference::recording::Recording;
use types::language::Language;
use types::rate::FrameRate;

const WHOLE: &str = "video_092_original";

#[test]
#[ignore = "reference"]
fn the_092_gives_the_fourteen_tracks_of_the_reference() {
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
    )
    .unwrap();
    let found = agreement::agreement(processed, &recording, rate);
    assert_eq!(found.processed.tracks.len(), 14);
    assert_eq!((found.reference, found.paired, found.agreeing), (14, 14, 14));
    assert_eq!((found.lost, found.added), (0, 0));
    assert_eq!(found.discrepancies, []);
}
