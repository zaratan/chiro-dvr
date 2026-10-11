use std::io::{self, Write};

use engine::input::{Error, Input};
use media::decode::Threads;
use reference::frames::Frames;
use reference::location::{recorded_video, recording_directories};
use reference::recording::SUMMARY_FILE;
use reference::video::VideoSummary;

#[test]
#[ignore = "local"]
fn the_twenty_six_recorded_videos_read_as_ffprobe_counted_them_without_refusal() {
    let directories = recording_directories().unwrap();
    assert_eq!(directories.len(), 26);
    let mut out = io::stdout().lock();
    for directory in directories {
        let summary = VideoSummary::read(&directory.join(SUMMARY_FILE)).unwrap();
        let probed = Frames::read(&directory.join(&summary.files.frames)).unwrap();
        let path = recorded_video(&summary.video.name).unwrap();
        let reading = Input::open(&path, Threads::Automatic)
            .unwrap()
            .read(|_, _| Ok::<(), Error>(()))
            .unwrap_or_else(|error| panic!("{error}"));
        let name = &summary.video.name;
        writeln!(
            out,
            "{name}: {} announced, {} read, {} in the reference, {:?}",
            reading.video().announced_frames(),
            reading.frames().len(),
            summary.counts.frames,
            reading.losses()
        )
        .unwrap();
        assert_eq!(reading.frames().len(), summary.counts.frames, "{name}");
        let losses = reading.losses();
        assert_eq!((losses.before_first, losses.after_last), (0, 0), "{name}");
        assert_eq!(
            losses.at_jumps,
            reading.video().announced_frames().get() - summary.counts.frames,
            "{name}"
        );
        assert_eq!(
            reading.video().frame_rate().get(),
            summary.stream.avg_frame_rate,
            "{name}"
        );
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
        assert!(read == expected, "{name}: timestamps or key frames differ from ffprobe");
    }
}
