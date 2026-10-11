use engine::input::{Error, Input};
use media::decode::Threads;
use reference::fixture::fixture;

#[test]
fn a_yuvj420p_video_is_refused_when_opened_and_named() {
    let path = fixture("full_range.mp4").unwrap();
    let refused = Input::open(&path, Threads::Automatic).unwrap_err();
    assert!(
        matches!(&refused, Error::FullRange { path: named } if *named == path),
        "{refused:?}"
    );
}

#[test]
fn a_video_that_turns_to_full_range_midway_is_refused_at_its_first_full_range_frame() {
    let path = fixture("full_range_midway.mp4").unwrap();
    let input = Input::open(&path, Threads::Automatic).unwrap();
    let mut visited = 0;
    let refused = input
        .read(|_, _| {
            visited += 1;
            Ok::<(), Error>(())
        })
        .unwrap_err();
    assert!(
        matches!(&refused, Error::FullRange { path: named } if *named == path),
        "{refused:?}"
    );
    assert_eq!(visited, 2);
}
