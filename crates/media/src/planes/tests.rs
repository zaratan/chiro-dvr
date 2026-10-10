use super::copy_rows;

#[test]
fn strided_rows_are_packed_without_their_padding() {
    let source = [1, 2, 0, 0, 3, 4, 0, 0];
    let mut packed = [9; 4];
    assert!(copy_rows(&source, 4, &mut packed, 2, 2, 2));
    assert_eq!(packed, [1, 2, 3, 4]);
}

#[test]
fn packed_rows_are_spread_over_a_wider_stride() {
    let mut strided = [9; 6];
    assert!(copy_rows(&[1, 2, 3, 4], 2, &mut strided, 3, 2, 2));
    assert_eq!(strided, [1, 2, 9, 3, 4, 9]);
}

#[test]
fn a_source_with_too_few_rows_is_reported() {
    let mut packed = [0; 4];
    assert!(!copy_rows(&[1, 2], 2, &mut packed, 2, 2, 2));
}

#[test]
fn a_row_wider_than_the_source_stride_is_reported() {
    let mut packed = [0; 4];
    assert!(!copy_rows(&[1, 2, 3], 1, &mut packed, 2, 2, 2));
}

#[test]
fn a_zero_stride_copies_nothing_instead_of_panicking() {
    let mut packed = [0; 4];
    assert!(!copy_rows(&[1, 2, 3, 4], 0, &mut packed, 2, 2, 2));
    assert!(copy_rows(&[], 0, &mut packed, 2, 2, 0));
}
