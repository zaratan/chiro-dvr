#[path = "support/sequential.rs"]
mod sequential;
#[path = "support/windows.rs"]
mod windows;

use render::source::{Origin, Source};
use types::frame::FrameIndex;
use types::span::Span;

const EXTRACT: &str = "video_092_original_3m24-4m05.mp4";
const WINDOWS: usize = 12;
const LONGEST: usize = 380;
const SEED: u64 = 110;

#[cfg(test)]
fn split_mix(state: &mut u64) -> usize {
    *state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
    let mut mixed = *state;
    mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
    usize::try_from((mixed ^ (mixed >> 31)) >> 32).unwrap()
}

#[test]
#[ignore = "slow"]
fn random_windows_of_the_extract_serve_the_frames_and_pixels_of_a_sequential_reading() {
    let path = reference::fixture::fixture(EXTRACT).unwrap();
    let read = sequential::read(&path);
    let frames = read.timeline.frame_count();
    let source = Source::new(&path, &read.timeline, &read.keys, &read.errors).unwrap();
    let mut state = SEED;
    for _ in 0..WINDOWS {
        let length = 1 + split_mix(&mut state) % LONGEST;
        let first = split_mix(&mut state) % (frames - length + 1);
        let window = Span::new(FrameIndex::new(first), FrameIndex::new(first + length - 1)).unwrap();
        let origin = windows::assert_served_like_a_sequential_reading(&source, &read, window);
        assert!(matches!(origin, Origin::Key(key) if key <= window.first() && read.keys.contains(&key)));
    }
}
