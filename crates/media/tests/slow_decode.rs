use media::decode::Decoder;
use media::frame::{ColorRange, YuvFrame};
use types::rational::Rational;

#[test]
#[ignore = "slow"]
fn the_extract_decodes_1230_frames_of_1440x1080_at_10000_333_limited_range() {
    let path = reference::fixture::fixture("video_092_original_3m24-4m05.mp4").unwrap();
    let mut decoder = Decoder::open(&path).unwrap();
    let stream = *decoder.stream();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut frames = 0;
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        assert_eq!(info.unreadable_packets(), 0);
        frames += 1;
    }
    assert_eq!(frames, 1230);
    assert_eq!(stream.frame_rate(), Rational::new(10_000, 333).unwrap());
    assert_eq!(
        (stream.width(), stream.height(), stream.range()),
        (1440, 1080, ColorRange::Limited)
    );
    assert_eq!(
        (frame.width(), frame.height(), frame.range()),
        (1440, 1080, ColorRange::Limited)
    );
}
