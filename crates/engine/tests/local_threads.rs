use std::io::{self, Write};
use std::num::NonZeroUsize;

use engine::damage::Damage;
use engine::input::{Error, Input};
use media::decode::{FrameInfo, Threads};
use media::fingerprint;
use reference::location::{Reference, recorded_video, recording_directories};
use reference::recording::SUMMARY_FILE;
use reference::video::VideoSummary;

const DAMAGED_VIDEOS: [&str; 3] = ["video_103.mp4", "video_122.mp4", "video_124.mp4"];

#[cfg(test)]
fn frames(name: &str, half_window: usize, threads: Threads) -> (Damage, Vec<(FrameInfo, u64)>) {
    let input = Input::open(&recorded_video(name).unwrap(), threads).unwrap();
    let mut frames = Vec::new();
    let reading = input
        .read(|info, frame| {
            frames.push((info, fingerprint::of(frame)));
            Ok::<(), Error>(())
        })
        .unwrap();
    (Damage::of(&reading, half_window), frames)
}

#[test]
#[ignore = "local"]
fn outside_damaged_spans_real_damaged_videos_do_not_depend_on_the_number_of_threads() {
    let mut out = io::stdout().lock();
    for directory in recording_directories(Reference::V011).unwrap() {
        let summary = VideoSummary::read(&directory.join(SUMMARY_FILE)).unwrap();
        let name = summary.video.name.as_str();
        if !DAMAGED_VIDEOS.contains(&name) {
            continue;
        }
        let half_window = summary.background_half_window;
        let (damage, automatic) = frames(name, half_window, Threads::Automatic);
        let (single_damage, single) = frames(name, half_window, Threads::Exactly(NonZeroUsize::MIN));
        assert_eq!(automatic.len(), single.len(), "{name}");
        assert_eq!(damage, single_damage, "{name}");
        let damaged = |info: &FrameInfo| damage.spans().iter().any(|span| span.contains(info.index()));
        let differing: Vec<usize> = automatic
            .iter()
            .zip(&single)
            .filter(|(a, b)| a != b)
            .map(|(a, _)| a.0.index().get())
            .collect();
        let inside = differing.iter().filter(|frame| damaged(&automatic[**frame].0)).count();
        writeln!(
            out,
            "{name}: {} frames differ, {inside} inside damaged spans",
            differing.len()
        )
        .unwrap();
        assert_eq!(inside, differing.len(), "{name}");
    }
}
