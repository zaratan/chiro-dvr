use super::Reach;
use crate::error::Error;
use crate::setting::Setting;

#[test]
fn a_track_reaches_the_margin_plus_the_way_it_should_have_flown() {
    assert_eq!(Reach::new(50.0, 1.0).unwrap().farthest(50.0), 100.0);
}

#[test]
fn a_fast_track_reaches_farther_than_a_slow_one() {
    let reach = Reach::default();
    assert!(reach.farthest(60.0) > 109.0);
    assert!(reach.farthest(22.5) < 109.0);
}

#[test]
fn without_travel_the_reach_is_the_margin_alone() {
    assert_eq!(Reach::new(30.0, 0.0).unwrap().farthest(1000.0), 30.0);
}

#[test]
fn the_margin_must_be_positive_and_the_travel_not_negative() {
    let refused = |margin, travel| match Reach::new(margin, travel) {
        Err(Error::Setting { setting, .. }) => Some(setting),
        _ => None,
    };
    assert_eq!(refused(0.0, 1.0), Some(Setting::ReachMargin));
    assert_eq!(refused(f64::INFINITY, 1.0), Some(Setting::ReachMargin));
    assert_eq!(refused(50.0, -0.1), Some(Setting::ReachTravel));
    assert_eq!(refused(50.0, f64::NAN), Some(Setting::ReachTravel));
    assert_eq!(refused(50.0, 0.0), None);
}
