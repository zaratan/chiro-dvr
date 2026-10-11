use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::num::NonZeroUsize;

use engine::config::DetectConfig;
use engine::damage::Damage;
use engine::detection::{Detected, detect_input};
use engine::input::{Error, Input};
use media::decode::{FrameInfo, Threads};
use rayon::ThreadPoolBuilder;
use reference::fixture::fixture;
use types::frame::FrameIndex;
use types::span::Span;

#[cfg(test)]
#[path = "support/slices.rs"]
mod slices;

const AUTOMATIC_WORKERS: usize = 0;
const CUT: [usize; 5] = [248, 555, 860, 861, 1222];
const ERROR_FRAMES: [usize; 5] = [248, 555, 860, 861, 1222];
const FFPROBE_ERROR_FRAMES: [usize; 5] = [248, 555, 860, 861, 1222];
const SPANS: [(usize, usize); 4] = [(248, 254), (555, 569), (860, 869), (1222, 1229)];

#[cfg(test)]
fn span(first: usize, last: usize) -> Span {
    Span::new(FrameIndex::new(first), FrameIndex::new(last)).unwrap()
}

#[cfg(test)]
fn canary(threads: Threads) -> (Damage, Vec<(FrameInfo, u64)>) {
    let path = fixture("video_092_original_3m24-4m05.mp4").unwrap();
    let input = Input::open(&path, threads)
        .unwrap()
        .with_seam(slices::cutting_the_last_third_of(BTreeSet::from(CUT)));
    let mut frames = Vec::new();
    let reading = input
        .read(|info, frame| {
            let mut hasher = DefaultHasher::new();
            (frame.y(), frame.u(), frame.v()).hash(&mut hasher);
            frames.push((info, hasher.finish()));
            Ok::<(), Error>(())
        })
        .unwrap();
    (Damage::of(&reading, 0).unwrap(), frames)
}

#[test]
#[ignore = "slow"]
fn the_cut_slices_of_the_extract_are_exactly_the_frames_in_error_and_contain_those_of_ffprobe() {
    let ffprobe: BTreeSet<usize> = BTreeSet::from(FFPROBE_ERROR_FRAMES);
    for threads in [Threads::Automatic, Threads::Exactly(NonZeroUsize::MIN)] {
        let (damage, frames) = canary(threads);
        let found: BTreeSet<usize> = damage.error_frames().iter().map(|frame| frame.get()).collect();
        assert_eq!(frames.len(), 1230, "{threads:?}");
        assert_eq!(found, BTreeSet::from(ERROR_FRAMES), "{threads:?}");
        assert!(found.is_superset(&ffprobe), "{threads:?}");
        assert_eq!(damage.jump_frames(), [], "{threads:?}");
        assert_eq!(
            damage.spans(),
            SPANS.map(|(first, last)| span(first, last)),
            "{threads:?}"
        );
    }
}

#[test]
#[ignore = "slow"]
fn outside_damaged_spans_the_frames_do_not_depend_on_the_number_of_threads() {
    let (damage, automatic) = canary(Threads::Automatic);
    let (_, single) = canary(Threads::Exactly(NonZeroUsize::MIN));
    let damaged = |frame: &FrameInfo| damage.spans().iter().any(|span| span.contains(frame.index()));
    let outside = |frames: &[(FrameInfo, u64)]| -> Vec<(FrameInfo, u64)> {
        frames.iter().filter(|(info, _)| !damaged(info)).copied().collect()
    };
    assert_eq!(automatic.len(), single.len());
    assert_eq!(outside(&automatic), outside(&single));
}

#[cfg(test)]
fn detected_canary(threads: Threads, workers: usize) -> Detected {
    let path = fixture("video_092_original_3m24-4m05.mp4").unwrap();
    let input = Input::open(&path, threads)
        .unwrap()
        .with_seam(slices::cutting_the_last_third_of(BTreeSet::from(CUT)));
    ThreadPoolBuilder::new()
        .num_threads(workers)
        .build()
        .unwrap()
        .install(|| detect_input(input, &DetectConfig::default()).unwrap())
}

#[test]
#[ignore = "slow"]
fn outside_damaged_spans_the_detections_do_not_depend_on_the_number_of_threads() {
    let automatic = detected_canary(Threads::Automatic, AUTOMATIC_WORKERS);
    let single = detected_canary(Threads::Exactly(NonZeroUsize::MIN), 1);
    let spans = automatic.damage().spans();
    let outside = |detected: &Detected| -> Vec<_> {
        detected
            .detections()
            .iter()
            .filter(|(frame, _)| !spans.iter().any(|span| span.contains(*frame)))
            .map(|(_, found)| found.to_vec())
            .collect()
    };

    assert_eq!(single.damage(), automatic.damage());
    assert_eq!(spans.len(), SPANS.len());
    assert_eq!(outside(&single), outside(&automatic));
    assert!(outside(&automatic).iter().map(Vec::len).sum::<usize>() > 300);
}
