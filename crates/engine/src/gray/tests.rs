use super::gray;

#[test]
fn the_measured_gray_follows_the_limited_range_one_to_two_and_a_half_levels_below_its_formula() {
    let below: Vec<f64> = (17..=236_u8)
        .map(|luma| (f64::from(luma) - 16.0) * 255.0 / 219.0 - f64::from(gray(luma)))
        .collect();

    assert!(below.iter().all(|&gap| (1.1..2.4).contains(&gap)), "{below:?}");
    assert!((0..u8::MAX).all(|luma| gray(luma) <= gray(luma + 1)));
    assert_eq!((gray(17), gray(18), gray(236), gray(237)), (0, 1, 254, 255));
}
