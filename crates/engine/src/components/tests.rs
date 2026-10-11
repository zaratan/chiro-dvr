use std::collections::BTreeSet;
use std::num::NonZeroU32;

use types::detection::Detection;
use types::frame::FrameIndex;
use types::size::SourceSize;

use super::Components;
use crate::closing::Closing;
use crate::config::DetectConfig;
use crate::geometry::Geometry;
use crate::synthetic::Noise;
use crate::threshold::Lit;

const WIDTH: usize = 320;
const HEIGHT: usize = 160;
const FRAME: FrameIndex = FrameIndex::new(7);

fn geometry() -> Geometry {
    let source = SourceSize::new(NonZeroU32::new(480).unwrap(), NonZeroU32::new(240).unwrap());
    Geometry::new(source, NonZeroU32::new(320).unwrap()).unwrap()
}

fn detected(lit: &[Lit]) -> Vec<Detection> {
    let geometry = geometry();
    let mut closing = Closing::new(geometry);
    let mut found = Vec::new();
    Components::new(geometry, &DetectConfig::default()).run(closing.run(lit), lit, FRAME, &mut found);
    found
}

fn square(column: usize, row: usize, size: usize, magnitude: f32) -> Vec<Lit> {
    (row..row + size)
        .flat_map(|row| (column..column + size).map(move |column| row * WIDTH + column))
        .map(|pixel| Lit {
            pixel: pixel as u32,
            magnitude,
        })
        .collect()
}

fn assert_close(found: &[Detection], expected: &[Detection]) {
    let fields = |detection: &Detection| {
        [
            detection.x,
            detection.y,
            detection.left,
            detection.top,
            detection.width,
            detection.height,
            detection.area,
            detection.amplitude,
        ]
    };
    assert_eq!(found.len(), expected.len());
    for (found, expected) in found.iter().zip(expected) {
        assert_eq!(found.frame, expected.frame);
        for (found, expected) in fields(found).into_iter().zip(fields(expected)) {
            assert!((found - expected).abs() < 1e-9, "{found} {expected}");
        }
    }
}

fn sorted(mut lit: Vec<Lit>) -> Vec<Lit> {
    lit.sort_by_key(|lit| lit.pixel);
    lit
}

fn many_blobs_with_bridged_pairs() -> Vec<Lit> {
    let mut noise = Noise::new(3);
    let mut lit = Vec::new();
    for row in 0..10 {
        for column in 0..20 {
            let (top, left) = (16 * row + usize::from(row != 0) * 3, 16 * column + 3);
            let size = if column % 5 == 0 {
                3
            } else {
                2 + (noise.uniform() * 3.0) as usize
            };
            let magnitude = (30.0 + 60.0 * noise.uniform()) as f32 * 64.0;
            lit.extend(square(left, top, size, magnitude));
            if column % 5 == 0 {
                lit.extend(square(left + size + 1, top, size, magnitude * 0.5));
            }
        }
    }
    sorted(lit)
}

fn one_by_one(closed: &[u32], lit: &[Lit]) -> Vec<Detection> {
    let closed: BTreeSet<usize> = closed.iter().map(|&pixel| pixel as usize).collect();
    let mut seen = BTreeSet::new();
    let scale = geometry().scale();
    let mut found = Vec::new();
    for &start in &closed {
        if !seen.insert(start) {
            continue;
        }
        let mut component = vec![start];
        let mut next = 0;
        while let Some(&pixel) = component.get(next) {
            next += 1;
            let (row, column) = ((pixel / WIDTH) as isize, (pixel % WIDTH) as isize);
            for (down, right) in [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)] {
                let (row, column) = (row + down, column + right);
                if (0..HEIGHT as isize).contains(&row) && (0..WIDTH as isize).contains(&column) {
                    let around = row as usize * WIDTH + column as usize;
                    if closed.contains(&around) && seen.insert(around) {
                        component.push(around);
                    }
                }
            }
        }
        let columns: Vec<usize> = component.iter().map(|pixel| pixel % WIDTH).collect();
        let rows: Vec<usize> = component.iter().map(|pixel| pixel / WIDTH).collect();
        let hits: Vec<f32> = lit
            .iter()
            .filter(|lit| component.contains(&(lit.pixel as usize)))
            .map(|lit| lit.magnitude)
            .collect();
        let (left, right) = (*columns.iter().min().unwrap(), *columns.iter().max().unwrap());
        let (top, bottom) = (*rows.iter().min().unwrap(), *rows.iter().max().unwrap());
        let mean = |values: &[usize]| values.iter().sum::<usize>() as f64 / values.len() as f64;
        let area = scale.area_to_source(hits.len() as f64);
        if (4.0..=2700.0).contains(&area) {
            found.push(Detection {
                frame: FRAME,
                x: scale.coordinate_to_source(mean(&columns)),
                y: scale.coordinate_to_source(mean(&rows)),
                left: scale.length_to_source(left as f64),
                top: scale.length_to_source(top as f64),
                width: scale.length_to_source((right - left + 1) as f64),
                height: scale.length_to_source((bottom - top + 1) as f64),
                area,
                amplitude: f64::from(hits.iter().copied().fold(0.0, f32::max)) / 64.0,
            });
        }
    }
    found
}

#[test]
fn many_blobs_are_measured_as_one_by_one_in_scanning_order() {
    let lit = many_blobs_with_bridged_pairs();
    let closed = Closing::new(geometry()).run(&lit).to_vec();

    let found = detected(&lit);

    assert_eq!(found.len(), 200);
    assert_close(&found, &one_by_one(&closed, &lit));
}

#[test]
fn a_blob_is_placed_boxed_and_measured_in_source_pixels() {
    let found = detected(&square(10, 20, 2, 800.0));

    assert_close(
        &found,
        &[Detection {
            frame: FRAME,
            x: 16.0,
            y: 31.0,
            left: 15.0,
            top: 30.0,
            width: 3.0,
            height: 3.0,
            area: 9.0,
            amplitude: 12.5,
        }],
    );
}

#[test]
fn blobs_outside_the_area_bounds_are_ignored() {
    let single = square(10, 20, 1, 800.0);
    let pair = sorted([square(10, 20, 1, 800.0), square(11, 20, 1, 800.0)].concat());
    let strip = (40..40 + 1200 - 34 * 34).map(|row| Lit {
        pixel: (row * WIDTH + 134) as u32,
        magnitude: 800.0,
    });
    let largest: Vec<Lit> = square(100, 40, 34, 800.0).into_iter().chain(strip).collect();
    let one_more = sorted([largest.clone(), square(135, 40, 1, 800.0)].concat());

    assert_eq!(detected(&single), []);
    assert_eq!(detected(&pair).len(), 1);
    assert!((detected(&sorted(largest))[0].area - 2700.0).abs() < 1e-9);
    assert_eq!(detected(&one_more), []);
}

#[test]
fn the_higher_blob_comes_first_then_the_left_one() {
    let lit = sorted(
        [
            square(200, 10, 2, 800.0),
            square(50, 40, 2, 800.0),
            square(10, 40, 2, 800.0),
        ]
        .concat(),
    );

    let found = detected(&lit);

    let lefts: Vec<f64> = found.iter().map(|found| found.left).collect();
    assert!(
        lefts
            .iter()
            .zip([300.0, 15.0, 75.0])
            .all(|(left, expected)| (left - expected).abs() < 1e-9)
            && lefts.len() == 3,
        "{lefts:?}"
    );
}

#[test]
fn one_frame_after_another_starts_from_unlabelled_pixels() {
    let geometry = geometry();
    let (mut closing, mut components) = (
        Closing::new(geometry),
        Components::new(geometry, &DetectConfig::default()),
    );
    let first = square(10, 20, 3, 800.0);
    let second = square(12, 22, 2, 900.0);
    let mut found = Vec::new();

    components.run(closing.run(&first), &first, FRAME, &mut found);
    found.clear();
    components.run(closing.run(&second), &second, FRAME, &mut found);

    assert_close(&found, &detected(&second));
}
