use std::collections::BTreeSet;
use std::io::{self, Write};

use engine::damage::Damage;
use engine::input::{Error, Input};
use media::decode::Threads;
use reference::location::{recorded_video, recording_directories};
use reference::recording::SUMMARY_FILE;
use reference::video::VideoSummary;
use types::frame::FrameIndex;
use types::span::Span;

struct Concealed {
    video: &'static str,
    frame: usize,
    damaged_frames: usize,
}

const CONCEALED_WITHOUT_ERROR_MESSAGE: [Concealed; 1] = [Concealed {
    video: "video_124.mp4",
    frame: 2197,
    damaged_frames: 7458,
}];

fn set(frames: &[FrameIndex]) -> BTreeSet<usize> {
    frames.iter().map(|frame| frame.get()).collect()
}

fn damaged_frames(spans: &[Span]) -> usize {
    spans.iter().map(|span| span.frame_count()).sum()
}

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_are_damaged_where_ffprobe_says_plus_frames_it_conceals_silently() {
    let directories = recording_directories().unwrap();
    assert_eq!(directories.len(), 26);
    let mut out = io::stdout().lock();
    for directory in directories {
        let summary = VideoSummary::read(&directory.join(SUMMARY_FILE)).unwrap();
        let name = summary.video.name.as_str();
        let path = recorded_video(name).unwrap();
        let reading = Input::open(&path, Threads::Automatic)
            .unwrap()
            .read(|_, _| Ok::<(), Error>(()))
            .unwrap();
        let damage = Damage::of(&reading, summary.background_half_window);
        let (found, probed) = (set(damage.error_frames()), set(&summary.ffprobe.error_frames));
        let extra: Vec<usize> = found.difference(&probed).copied().collect();
        let (rust, recorded) = (damaged_frames(damage.spans()), damaged_frames(&summary.damaged_spans));
        writeln!(
            out,
            "{name}: {} frames in error ({} for ffprobe), extra {extra:?}, {rust} damaged frames ({recorded})",
            found.len(),
            probed.len()
        )
        .unwrap();
        assert!(found.is_superset(&probed), "{name}");
        assert_eq!(damage.jump_frames(), summary.ffprobe.gap_frames, "{name}");
        let concealed: Vec<&Concealed> = CONCEALED_WITHOUT_ERROR_MESSAGE
            .iter()
            .filter(|concealed| concealed.video == name)
            .collect();
        assert_eq!(
            extra,
            concealed.iter().map(|concealed| concealed.frame).collect::<Vec<_>>(),
            "{name}"
        );
        match concealed.first() {
            None => assert_eq!(damage.spans(), summary.damaged_spans, "{name}"),
            Some(concealed) => {
                assert_eq!(rust, concealed.damaged_frames, "{name}");
                assert!(summary.damaged_spans.iter().all(|recorded| {
                    damage
                        .spans()
                        .iter()
                        .any(|span| span.contains(recorded.first()) && span.contains(recorded.last()))
                }));
            }
        }
    }
}
