use std::collections::BTreeSet;
use std::io::{self, Write};

use engine::damage::Damage;
use engine::input::{Error, Input};
use media::decode::Threads;
use reference::location::{Reference, recorded_video, recording_directories};
use reference::recording::SUMMARY_FILE;
use reference::video::VideoSummary;
use types::frame::FrameIndex;
use types::span::Span;

struct Concealed {
    video: &'static str,
    frames: &'static [usize],
    damaged_frames: usize,
}

const CONCEALED_WITHOUT_ERROR_MESSAGE: [Concealed; 1] = [Concealed {
    video: "video_124.mp4",
    frames: &[2197],
    damaged_frames: 7458,
}];

const FLAGGED_ONLY_BY_SOME_SYSTEMS: [Concealed; 8] = [
    Concealed {
        video: "video_093.mp4",
        frames: &[1307, 7395],
        damaged_frames: 449,
    },
    Concealed {
        video: "video_097.mp4",
        frames: &[5444],
        damaged_frames: 535,
    },
    Concealed {
        video: "video_120.mp4",
        frames: &[4589],
        damaged_frames: 2215,
    },
    Concealed {
        video: "video_121.mp4",
        frames: &[8844],
        damaged_frames: 4912,
    },
    Concealed {
        video: "video_122.mp4",
        frames: &[2264],
        damaged_frames: 6073,
    },
    Concealed {
        video: "video_123.mp4",
        frames: &[330, 1250, 2267, 3986, 4902, 5417],
        damaged_frames: 6627,
    },
    Concealed {
        video: "video_124.mp4",
        frames: &[2197, 3643],
        damaged_frames: 7458,
    },
    Concealed {
        video: "video_128.mp4",
        frames: &[2830],
        damaged_frames: 1006,
    },
];

fn expected(name: &str, extra: &[usize]) -> Option<&'static Concealed> {
    let concealed = CONCEALED_WITHOUT_ERROR_MESSAGE
        .iter()
        .find(|concealed| concealed.video == name);
    let silent = concealed.map_or(&[][..], |concealed| concealed.frames);
    let flagged = FLAGGED_ONLY_BY_SOME_SYSTEMS
        .iter()
        .find(|flagged| flagged.video == name);
    if extra == silent {
        concealed
    } else {
        flagged.or(concealed)
    }
}

fn set(frames: &[FrameIndex]) -> BTreeSet<usize> {
    frames.iter().map(|frame| frame.get()).collect()
}

fn damaged_frames(spans: &[Span]) -> usize {
    spans.iter().map(|span| span.frame_count()).sum()
}

#[test]
#[ignore = "local"]
fn the_twenty_six_videos_are_damaged_where_ffprobe_says_plus_frames_it_conceals_silently_or_some_systems_flag() {
    let directories = recording_directories(Reference::V011).unwrap();
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
        let probe = summary.ffprobe.as_ref().unwrap();
        let (found, probed) = (set(damage.error_frames()), set(&probe.error_frames));
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
        assert_eq!(damage.jump_frames(), probe.gap_frames, "{name}");
        let concealed = expected(name, &extra);
        assert_eq!(extra, concealed.map_or(&[][..], |concealed| concealed.frames), "{name}");
        match concealed {
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
