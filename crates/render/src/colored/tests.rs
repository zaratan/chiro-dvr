use draw::color::PALETTE;

use super::{Colored, Mismatch};
use crate::ink::Ink;
use crate::testing::track;

#[test]
fn tracks_without_colours_all_keep_the_red_box() {
    let tracks = [track(1, &[(0, 10.0, 10.0)]), track(2, &[(0, 20.0, 20.0)])];
    let colored = Colored::new(&tracks, &[]).unwrap();
    assert_eq!(colored.inks(), [Ink::of(None), Ink::of(None)]);
}

#[test]
fn each_track_takes_the_colour_of_its_rank() {
    let tracks = [track(1, &[(0, 10.0, 10.0)]), track(2, &[(0, 20.0, 20.0)])];
    let colors = [None, Some(PALETTE[3])];
    let colored = Colored::new(&tracks, &colors).unwrap();
    assert_eq!(colored.inks(), [Ink::of(None), Ink::of(Some(PALETTE[3]))]);
}

#[test]
fn colours_fewer_or_more_than_the_tracks_are_refused_rather_than_drawn_red_in_silence() {
    let two = [track(1, &[(0, 10.0, 10.0)]), track(2, &[(0, 20.0, 20.0)])];
    for (tracks, count) in [(&two[..], 1), (&two[..], 3), (&[][..], 1)] {
        let colors = vec![Some(PALETTE[0]); count];
        assert_eq!(
            Colored::new(tracks, &colors).unwrap_err(),
            Mismatch {
                colors: count,
                tracks: tracks.len(),
            }
        );
    }
}

#[test]
fn a_video_without_tracks_takes_no_colour() {
    assert_eq!(Colored::new(&[], &[]).unwrap().inks(), []);
}
