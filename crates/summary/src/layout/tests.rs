use std::num::NonZeroU32;

use draw::style::style_for;
use draw::text::Typeface;
use types::frame::Pts;
use types::rate::FrameRate;
use types::rational::Rational;
use types::timeline::Timeline;

use super::{Summary, layout};
use crate::placement::Frame;
use crate::testing::track;

#[test]
fn the_legend_time_of_a_track_comes_from_the_pts_of_its_first_frame() {
    let rate = FrameRate::new(Rational::new(10, 1).unwrap()).unwrap();
    let late = |frame: i64| Some(Pts::new(if frame < 5 { frame } else { frame + 600 }));
    let timeline = Timeline::new((0..20).map(late).collect(), Rational::new(1, 10).unwrap(), rate).unwrap();
    let tracks = [track(1, 10, &[[10.0, 10.0], [60.0, 10.0], [110.0, 10.0]])];
    let typeface = Typeface::roboto().unwrap();
    let style = style_for(NonZeroU32::new(320).unwrap(), "1", &typeface);
    let summary = Summary {
        name: "v",
        tracks: &tracks,
        timeline: &timeline,
        ignored: &[],
        damaged: &[],
    };
    let frame = Frame {
        width: 320.0,
        height: 240.0,
    };
    let pages = layout(&summary, frame, &style);
    assert_eq!(pages[0].marks[0].start, "1:01");
}
