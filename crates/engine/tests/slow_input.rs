use engine::input::{Error, Input};
use media::decode::Threads;
use reference::fixture::fixture;
use reference::frames::Frames;
use reference::location::committed;
use reference::recording::SUMMARY_FILE;
use reference::video::VideoSummary;

#[test]
#[ignore = "slow"]
fn the_extract_reads_its_1230_frames_with_the_timestamps_and_key_frames_of_ffprobe() {
    let recorded = committed("video_092_original_3m24-4m05");
    let summary = VideoSummary::read(&recorded.join(SUMMARY_FILE)).unwrap();
    let probed = Frames::read(&recorded.join(&summary.files.frames)).unwrap();
    let path = fixture("video_092_original_3m24-4m05.mp4").unwrap();
    let reading = Input::open(&path, Threads::Automatic)
        .unwrap()
        .read(|_, _| Ok::<(), Error>(()))
        .unwrap();
    assert_eq!(reading.frames().len(), 1230);
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
}
