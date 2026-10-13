use types::frame::FrameIndex;

use super::{Class, Error, Motion, Point, Target, UnknownMotion};

fn class() -> Class {
    Class::new(-30.0, 3.0, Motion::Pass).unwrap()
}

#[test]
fn each_motion_reads_back_from_its_name() {
    for motion in Motion::ALL {
        assert_eq!(motion.name().parse::<Motion>(), Ok(motion));
        assert_eq!(motion.to_string(), motion.name());
    }
}

#[test]
fn an_unknown_motion_is_refused_with_the_known_ones() {
    let refused = "glide".parse::<Motion>().unwrap_err();

    assert_eq!(refused, UnknownMotion("glide".to_owned()));
    assert_eq!(
        refused.to_string(),
        "unknown motion \"glide\", expected pass, hunt or circle"
    );
}

#[test]
fn a_class_needs_a_finite_amplitude_and_a_positive_sigma() {
    assert_eq!(
        Class::new(f64::NAN, 3.0, Motion::Hunt).unwrap_err().to_string(),
        "the amplitude of a target must be a finite number, not NaN"
    );
    for sigma in [0.0, -1.0, f64::INFINITY] {
        assert_eq!(Class::new(-15.0, sigma, Motion::Hunt), Err(Error::Sigma(sigma)));
    }
    let taken = Class::new(-15.0, 1.5, Motion::Circle).unwrap();
    assert_eq!(
        (taken.amplitude(), taken.sigma(), taken.motion()),
        (-15.0, 1.5, Motion::Circle)
    );
}

#[test]
fn a_target_needs_one_amplitude_per_position_and_at_least_one_position() {
    let at = Point { x: 1.0, y: 2.0 };

    assert_eq!(
        Target::new(7, FrameIndex::new(0), vec![at; 2], vec![-30.0], class()),
        Err(Error::Lengths {
            id: 7,
            positions: 2,
            amplitudes: 1
        })
    );
    assert_eq!(
        Target::new(7, FrameIndex::new(0), Vec::new(), Vec::new(), class()),
        Err(Error::Empty { id: 7 })
    );
}

#[test]
fn a_target_is_seen_from_its_start_to_its_end_frame_included() {
    let positions = vec![Point { x: 1.0, y: 2.0 }, Point { x: 3.0, y: 4.0 }];
    let target = Target::new(2, FrameIndex::new(5), positions.clone(), vec![-20.0, -25.0], class()).unwrap();

    assert_eq!(
        (target.id(), target.start(), target.end()),
        (2, FrameIndex::new(5), FrameIndex::new(6))
    );
    assert_eq!(target.position(FrameIndex::new(4)), None);
    assert_eq!(target.position(FrameIndex::new(6)), Some(positions[1]));
    assert_eq!(target.amplitude_at(FrameIndex::new(5)), Some(-20.0));
    assert_eq!(target.amplitude_at(FrameIndex::new(7)), None);
}

#[test]
fn a_target_whose_last_frame_cannot_be_numbered_is_refused() {
    let positions = vec![Point { x: 1.0, y: 2.0 }; 2];

    assert_eq!(
        Target::new(5, FrameIndex::new(usize::MAX), positions, vec![-20.0; 2], class()),
        Err(Error::TooLong { id: 5 })
    );
}
