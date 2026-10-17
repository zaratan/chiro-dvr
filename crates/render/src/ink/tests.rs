use draw::color::{BOX, INK, PALETTE, TRAIL};

use super::{Ink, ink_at, inks};

#[test]
fn a_coloured_track_draws_its_box_and_trail_in_its_colour_with_a_dark_rim() {
    assert_eq!(
        Ink::of(Some(PALETTE[1])),
        Ink {
            mark: PALETTE[1],
            trail: PALETTE[1],
            rim: Some(INK),
        }
    );
}

#[test]
fn a_track_without_a_colour_keeps_the_red_box_and_yellow_trail_without_a_rim() {
    assert_eq!(
        Ink::of(None),
        Ink {
            mark: BOX,
            trail: TRAIL,
            rim: None,
        }
    );
}

#[test]
fn an_index_beyond_the_inks_falls_back_to_the_red_box() {
    let found = inks(&[Some(PALETTE[2])], 1);
    assert_eq!(ink_at(&found, 0), Ink::of(Some(PALETTE[2])));
    assert_eq!(ink_at(&found, 1), Ink::of(None));
}

#[test]
fn tracks_beyond_the_given_colours_fall_back_to_the_red_box() {
    let found = inks(&[Some(PALETTE[0]), None], 3);
    assert_eq!(found, [Ink::of(Some(PALETTE[0])), Ink::of(None), Ink::of(None)]);
}
