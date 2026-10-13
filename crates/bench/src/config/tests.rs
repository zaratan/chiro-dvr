use super::{Error, MatchConfig, classes};
use crate::synthetic::{self, Motion};

#[test]
fn the_default_match_is_24_pixels_at_half_purity() {
    assert_eq!(MatchConfig::default(), MatchConfig::new(24.0, 0.5).unwrap());
    assert_eq!(
        (MatchConfig::default().radius(), MatchConfig::default().purity()),
        (24.0, 0.5)
    );
}

#[test]
fn a_radius_must_be_positive_and_a_purity_in_zero_excluded_to_one_included() {
    assert!(MatchConfig::new(0.1, 1.0).is_ok());
    for radius in [0.0, -1.0, f64::NAN, f64::INFINITY] {
        assert!(
            matches!(MatchConfig::new(radius, 0.5), Err(Error::Radius(_))),
            "{radius}"
        );
    }
    for purity in [0.0, 1.01, f64::NAN] {
        assert!(
            matches!(MatchConfig::new(24.0, purity), Err(Error::Purity(_))),
            "{purity}"
        );
    }
    assert_eq!(
        Error::Purity(0.0).to_string(),
        "the match purity must be in ]0, 1], not 0"
    );
}

#[test]
fn classes_come_motion_first_in_the_known_order_then_amplitude_then_sigma() {
    let built = classes(&[Motion::Circle, Motion::Hunt], &[-60.0, -30.0], &[3.0, 1.5]).unwrap();

    let keys: Vec<_> = built
        .iter()
        .map(|class| (class.motion(), class.amplitude(), class.sigma()))
        .collect();
    assert_eq!(
        keys,
        [
            (Motion::Hunt, -60.0, 3.0),
            (Motion::Hunt, -60.0, 1.5),
            (Motion::Hunt, -30.0, 3.0),
            (Motion::Hunt, -30.0, 1.5),
            (Motion::Circle, -60.0, 3.0),
            (Motion::Circle, -60.0, 1.5),
            (Motion::Circle, -30.0, 3.0),
            (Motion::Circle, -30.0, 1.5),
        ]
    );
}

#[test]
fn a_class_of_invalid_sigma_is_refused() {
    assert_eq!(
        classes(&[Motion::Pass], &[-15.0], &[0.0]),
        Err(synthetic::Error::Sigma(0.0))
    );
}
