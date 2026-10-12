use super::{Crossings, Frame, KeepClear, MARKER_GAP, Placer, candidates, collisions, place_markers};
use crate::geometry::distance;
use crate::testing::path;

const SIZE: Frame = Frame {
    width: 400.0,
    height: 300.0,
};
const RADIUS: f64 = 10.0;
const NOTHING: KeepClear<'static> = KeepClear {
    spots: &[],
    radius: 0.0,
};

#[test]
fn marker_stays_whole_inside_the_frame_for_a_track_along_the_edge() {
    let centers = place_markers(&[path(&[[399.0, 0.0], [399.0, 299.0]])], SIZE, RADIUS, NOTHING);
    let [x, y] = centers[0];
    let inset = RADIUS + MARKER_GAP;
    assert!((inset..=SIZE.width - inset).contains(&x));
    assert!((inset..=SIZE.height - inset).contains(&y));
}

#[test]
fn markers_of_close_parallel_tracks_do_not_overlap() {
    let paths = [
        path(&[[50.0, 100.0], [350.0, 100.0]]),
        path(&[[50.0, 105.0], [350.0, 105.0]]),
    ];
    let centers = place_markers(&paths, SIZE, RADIUS, NOTHING);
    assert!(distance(centers[0], centers[1]) >= 2.0 * RADIUS + MARKER_GAP);
}

#[test]
fn marker_moves_off_a_crossing_that_sits_on_its_preferred_spot() {
    let crossing_at_fifth = [
        path(&[[50.0, 150.0], [350.0, 150.0]]),
        path(&[[110.0, 50.0], [110.0, 250.0]]),
    ];
    let centers = place_markers(&crossing_at_fifth, SIZE, RADIUS, NOTHING);
    assert!(distance(centers[0], [110.0, 150.0]) >= 2.0 * RADIUS);
}

#[test]
fn marker_sits_near_a_quarter_of_the_track_rather_than_at_its_very_start() {
    let centers = place_markers(&[path(&[[0.0, 150.0], [400.0, 150.0]])], SIZE, RADIUS, NOTHING);
    assert_eq!(centers[0][0], 100.0);
}

#[test]
fn marker_keeps_clear_of_arrow_heads() {
    let head = KeepClear {
        spots: &[[100.0, 150.0]],
        radius: 40.0,
    };
    let centers = place_markers(&[path(&[[0.0, 150.0], [400.0, 150.0]])], SIZE, RADIUS, head);
    assert!(distance(centers[0], [100.0, 150.0]) >= 40.0);
}

#[test]
fn single_point_track_gets_its_marker_on_that_point() {
    assert_eq!(
        place_markers(&[path(&[[60.0, 70.0]])], SIZE, RADIUS, NOTHING),
        [[60.0, 70.0]]
    );
}

#[test]
fn superimposed_tracks_get_separate_markers_the_same_way_every_time() {
    let same = [
        path(&[[50.0, 150.0], [350.0, 150.0]]),
        path(&[[50.0, 150.0], [350.0, 150.0]]),
    ];
    let first = place_markers(&same, SIZE, RADIUS, NOTHING);
    assert_eq!(first, place_markers(&same, SIZE, RADIUS, NOTHING));
    assert!(distance(first[0], first[1]) >= 2.0 * RADIUS + MARKER_GAP);
}

#[test]
fn a_frame_narrower_than_a_marker_still_gives_a_position_rather_than_failing() {
    let tiny = Frame {
        width: 10.0,
        height: 10.0,
    };
    let centers = place_markers(&[path(&[[0.0, 5.0], [10.0, 5.0]])], tiny, RADIUS, NOTHING);
    assert!(centers[0].iter().all(|coordinate| coordinate.is_finite()));
}

#[test]
fn a_marker_with_no_room_on_its_track_moves_just_beside_it_rather_than_covering_another() {
    let same_spot = [path(&[[200.0, 150.0]]), path(&[[200.0, 150.0]])];
    let centers = place_markers(&same_spot, SIZE, RADIUS, NOTHING);
    assert!(distance(centers[0], centers[1]) >= 2.0 * RADIUS + MARKER_GAP);
    let farthest = distance(centers[0], [200.0, 150.0]).max(distance(centers[1], [200.0, 150.0]));
    assert!(farthest <= 2.0 * RADIUS + MARKER_GAP + RADIUS / 2.0, "{farthest}");
}

#[test]
fn markers_crowded_into_a_corner_leave_it_rather_than_pile_up_on_the_edge() {
    let corner: Vec<_> = (0..6).map(|_| path(&[[399.0, 299.0]])).collect();
    let centers = place_markers(&corner, SIZE, RADIUS, NOTHING);
    for (index, center) in centers.iter().enumerate() {
        for other in &centers[index + 1..] {
            assert!(distance(*center, *other) >= 2.0 * RADIUS, "{centers:?}");
        }
    }
}

#[test]
fn a_marker_that_only_touches_its_own_arrow_head_stays_on_its_track() {
    let short = path(&[[100.0, 150.0], [120.0, 150.0]]);
    let head = KeepClear {
        spots: &[[120.0, 150.0]],
        radius: 50.0,
    };
    let centers = place_markers(&[short], SIZE, RADIUS, head);
    assert_eq!(centers[0][1], 150.0);
}

#[test]
fn a_repair_pass_separates_markers_left_on_top_of_each_other() {
    let paths = [
        path(&[[50.0, 150.0], [350.0, 150.0]]),
        path(&[[50.0, 160.0], [350.0, 160.0]]),
    ];
    let sampled: Vec<_> = paths.iter().map(|one| one.resampled(RADIUS / 2.0)).collect();
    let placer = Placer {
        candidates: paths.iter().map(|one| candidates(one, SIZE, RADIUS)).collect(),
        crossings: Crossings::new(&sampled, 2.0 * RADIUS),
        frame: SIZE,
        radius: RADIUS,
        keep_clear: NOTHING,
    };
    let mut centers = vec![[125.0, 155.0], [125.0, 155.0]];
    placer.repair(&[0, 1], &mut centers);
    assert_eq!(collisions(centers[0], &centers[1..], RADIUS, NOTHING), (0, 0));
}
