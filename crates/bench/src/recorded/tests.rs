use reference::bench::TargetLine;
use types::frame::FrameIndex;

use super::{Error, injected, target};
use crate::synthetic::injection::Observation;
use crate::synthetic::{self, Motion, Point, UnknownMotion};

fn line(motion: &str, amplitudes: Vec<f64>) -> TargetLine {
    TargetLine {
        id: 4,
        motion: motion.to_owned(),
        amplitude: -30.0,
        sigma: 3.0,
        start_frame: 120,
        positions: vec![[10.5, 20.0], [12.0, 21.5]],
        amplitudes,
        effective: Vec::new(),
        patch: None,
    }
}

#[test]
fn a_recorded_target_keeps_its_number_start_class_positions_and_amplitudes() {
    let read = target(&line("circle", vec![-20.0, -25.0])).unwrap();

    assert_eq!((read.id(), read.start()), (4, FrameIndex::new(120)));
    assert_eq!(
        (read.class().amplitude(), read.class().sigma(), read.class().motion()),
        (-30.0, 3.0, Motion::Circle)
    );
    assert_eq!(
        read.positions(),
        [Point { x: 10.5, y: 20.0 }, Point { x: 12.0, y: 21.5 }]
    );
    assert_eq!(read.amplitudes(), [-20.0, -25.0]);
}

#[test]
fn a_recorded_target_of_unknown_motion_is_refused_with_its_number() {
    let refused = target(&line("glide", vec![-20.0, -25.0])).unwrap_err();

    assert_eq!(
        refused,
        Error::Motion {
            id: 4,
            source: UnknownMotion("glide".to_owned())
        }
    );
    assert_eq!(refused.to_string(), "recorded target 4 has an unknown motion");
}

#[test]
fn a_recorded_target_missing_an_amplitude_is_refused_with_its_number() {
    let refused = target(&line("pass", vec![-20.0])).unwrap_err();

    assert!(
        matches!(
            refused,
            Error::Target {
                id: 4,
                source: synthetic::Error::Lengths { .. }
            }
        ),
        "{refused:?}"
    );
}

#[test]
fn a_recorded_contrast_is_observed_at_the_position_of_its_frame() {
    let mut recorded = line("hunt", vec![-20.0, -25.0]);
    recorded.effective = vec![(121, -17.0)];

    let read = injected(&recorded).unwrap();

    assert_eq!(
        read.observations,
        [Observation {
            frame: FrameIndex::new(121),
            at: Point { x: 12.0, y: 21.5 },
            effective: -17.0
        }]
    );
}

#[test]
fn a_recorded_contrast_outside_the_flight_of_its_target_is_refused() {
    let mut recorded = line("hunt", vec![-20.0, -25.0]);
    recorded.effective = vec![(122, -17.0)];

    assert_eq!(injected(&recorded), Err(Error::Observation { id: 4, frame: 122 }));
}

#[test]
fn an_injected_target_written_as_a_line_reads_back_the_same() {
    let mut recorded = line("circle", vec![-20.0, -25.0]);
    recorded.effective = vec![(120, -12.0), (121, -17.0)];
    let placed = injected(&recorded).unwrap();

    let written = super::line(&placed);

    assert_eq!(written, recorded);
    assert_eq!(injected(&written).unwrap(), placed);
}

#[test]
fn recorded_contrasts_out_of_frame_order_or_repeated_are_refused() {
    for effective in [vec![(121, -17.0), (120, -12.0)], vec![(120, -12.0), (120, -13.0)]] {
        let mut recorded = line("hunt", vec![-20.0, -25.0]);
        recorded.effective = effective;

        assert!(matches!(injected(&recorded), Err(Error::Order { id: 4, .. })));
    }
}

#[test]
fn a_targets_file_naming_one_target_twice_is_refused() {
    let lines = [line("pass", vec![-20.0, -25.0]), line("hunt", vec![-20.0, -25.0])];

    assert_eq!(super::targets(&lines), Err(Error::Duplicate { id: 4 }));
    assert_eq!(super::all_injected(&lines), Err(Error::Duplicate { id: 4 }));
    assert_eq!(super::targets(&lines[..1]).unwrap().len(), 1);
    assert_eq!(super::all_injected(&lines[..1]).unwrap().len(), 1);
}
