use std::path::PathBuf;

use draw::canvas::PixelBox;
use draw::color::PALETTE;
use engine::settings::Region;
use types::detection::Detection;
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
        left: 9.0,
        top: 9.0,
        width: 2.0,
        height: 2.0,
    };
    Track::new(id, vec![point]).unwrap()
}

#[test]
fn each_track_takes_the_colour_painted_under_its_number_and_none_when_it_is_not_on_the_summary() {
    let drawn = Drawn {
        pages: Vec::new(),
        painted: vec![
            Drawing {
                track: 7,
                color: PALETTE[3],
                page: PathBuf::from("resume.png"),
            },
            Drawing {
                track: 2,
                color: PALETTE[1],
                page: PathBuf::from("resume.png"),
            },
        ],
    };
    let tracks = [seen_once(2), seen_once(5), seen_once(7)];
    assert_eq!(drawn.colors(&tracks), [Some(PALETTE[1]), None, Some(PALETTE[3])]);
}
