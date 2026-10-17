use std::path::PathBuf;

use draw::canvas::PixelBox;
use draw::color::{PALETTE, Rgb};
use engine::settings::Region;
use summary::Painted;
use types::detection::{Detection, Strength};
use types::frame::FrameIndex;
use types::track::Track;

use super::{Drawing, Drawn, display_boxes};

#[test]
fn a_display_region_is_painted_from_its_rounded_down_start_to_its_rounded_up_end_included() {
    let band = Region {
        x0: 0.15,
        y0: 0.0,
        x1: 0.35,
        y1: 0.07,
    };

    assert_eq!(
        display_boxes(&[band], 1440, 1080),
        [PixelBox {
            left: 216,
            top: 0,
            right: 503,
            bottom: 75,
        }]
    );
}

#[test]
fn without_display_region_nothing_is_painted() {
    assert_eq!(display_boxes(&[], 1440, 1080), []);
}

fn seen_once(id: usize) -> Track {
    let point = Detection {
        frame: FrameIndex::new(0),
        x: 10.0,
        y: 10.0,
        area: 4.0,
        amplitude: 50.0,
        strength: Strength::Strong,
        left: 9.0,
        top: 9.0,
        width: 2.0,
        height: 2.0,
    };
    Track::new(id, vec![point]).unwrap()
}

fn drawing(track: usize, color: Rgb) -> Drawing {
    Drawing {
        page: PathBuf::from("resume.png"),
        painted: Painted {
            track,
            color,
            path: vec![[10.0, 10.0]],
            head: None,
            dot: [10.0, 10.0],
            anchor: None,
        },
    }
}

#[test]
fn each_track_takes_the_colour_painted_under_its_number_and_none_when_it_is_not_on_the_summary() {
    let drawn = Drawn {
        background: PathBuf::from("fond.png"),
        sheets: Vec::new(),
        painted: vec![drawing(7, PALETTE[3]), drawing(2, PALETTE[1])],
    };
    let tracks = [seen_once(2), seen_once(5), seen_once(7)];
    assert_eq!(drawn.colors(&tracks), [Some(PALETTE[1]), None, Some(PALETTE[3])]);
}
