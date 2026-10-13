use bench::recorded;
use bench::synthetic::injection::{gray_change, inject, window};
use engine::gray::gray;
use engine::luma::LumaMut;
use reference::bench::{TARGETS_FILE, read_targets};
use reference::location::recorded_bench;
use types::frame::FrameIndex;

const WIDTH: usize = 1440;
const HEIGHT: usize = 1080;
const BACKGROUNDS: std::ops::RangeInclusive<u8> = 80..=215;

#[derive(Debug, Default)]
struct Tally {
    patches: usize,
    without_patch: usize,
    pixels: usize,
    compared: usize,
    beyond_one_level: usize,
    floors_differing: usize,
    offset: i64,
}

#[cfg(test)]
fn tally(name: &str, plane: &mut [u8]) -> Tally {
    let lines = read_targets(&recorded_bench(name).unwrap().join(TARGETS_FILE)).unwrap();
    let mut tally = Tally::default();
    for line in &lines {
        let Some(patch) = &line.patch else {
            tally.without_patch += 1;
            continue;
        };
        let target = recorded::target(line).unwrap();
        let frame = FrameIndex::new(patch.frame);
        let center = target.position(frame).unwrap();
        let amplitude = target.amplitude_at(frame).unwrap();
        let sigma = target.class().sigma();
        let area = window(center, sigma, WIDTH, HEIGHT).unwrap();
        assert_eq!((area.left, area.top), (patch.left, patch.top), "target {}", line.id);
        assert_eq!(area.bottom - area.top, patch.values.len(), "target {}", line.id);
        tally.patches += 1;
        for (row, values) in patch.values.iter().enumerate() {
            assert_eq!(area.right - area.left, values.len(), "target {}", line.id);
            for (column, &value) in values.iter().enumerate() {
                tally.pixels += 1;
                let floor = gray_change(amplitude, sigma, center, area.left + column, area.top + row);
                tally.floors_differing += usize::from(floor as i16 != value);
            }
        }
        for background in BACKGROUNDS {
            for row in area.top..area.bottom {
                plane[row * WIDTH + area.left..row * WIDTH + area.right].fill(background);
            }
            inject(&mut LumaMut::new(plane, WIDTH).unwrap(), &target, frame).unwrap();
            for (row, values) in patch.values.iter().enumerate() {
                for (column, &value) in values.iter().enumerate() {
                    let after = plane[(area.top + row) * WIDTH + area.left + column];
                    let change = i64::from(gray(after)) - i64::from(gray(background));
                    let gap = change - i64::from(value);
                    tally.compared += 1;
                    tally.offset += gap;
                    tally.beyond_one_level += usize::from(gap.abs() > 1);
                }
            }
        }
    }
    tally
}

#[test]
#[ignore = "local"]
fn every_recorded_patch_is_injected_within_one_gray_level_and_without_bias_on_every_background_of_the_limited_range() {
    let mut plane = vec![0; WIDTH * HEIGHT];
    let transit = tally("transit", &mut plane);
    let hunt = tally("hunt", &mut plane);

    println!("transit: {transit:?}");
    println!("hunt: {hunt:?}");
    let compared = transit.compared + hunt.compared;
    let bias = (transit.offset + hunt.offset) as f64 / compared as f64;
    println!("mean offset to the recorded patches: {bias:.4}");
    assert_eq!(
        (
            transit.patches + transit.without_patch,
            hunt.patches + hunt.without_patch
        ),
        (270, 720)
    );
    assert_eq!(transit.patches + hunt.patches, 986);
    assert_eq!(transit.pixels + hunt.pixels, 471_046);
    assert_eq!(transit.floors_differing + hunt.floors_differing, 0);
    assert_eq!(transit.beyond_one_level + hunt.beyond_one_level, 0);
    assert!(bias.abs() < 0.01, "{bias}");
}
