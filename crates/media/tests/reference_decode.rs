use media::decode::Decoder;
use media::frame::{ColorRange, YuvFrame};
use types::rational::Rational;

#[test]
#[ignore = "reference"]
fn the_whole_092_decodes_its_9011_frames_at_its_average_rate() {
    let path = reference::fixture::fixture("video_092_original.mp4").unwrap();
    let mut decoder = Decoder::open(&path).unwrap();
    let stream = *decoder.stream();
    let mut frame = YuvFrame::new(1, 1, ColorRange::Unspecified).unwrap();
    let mut frames = 0;
    while let Some(info) = decoder.next(&mut frame).unwrap() {
        assert_eq!(info.unreadable_packets(), 0);
        frames += 1;
    }
    assert_eq!(frames, 9011);
    assert_eq!(stream.frame_rate(), Rational::new(90_110_000, 3_000_927).unwrap());
    assert_eq!(stream.time_base(), Rational::new(1, 10_000).unwrap());
    assert_eq!(
        (stream.width(), stream.height(), stream.range()),
        (1440, 1080, ColorRange::Limited)
    );
}
