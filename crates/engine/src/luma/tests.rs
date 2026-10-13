use super::{Error, LumaMut};

#[test]
fn a_plane_of_whole_rows_is_taken_and_written_row_by_row() {
    let mut pixels = vec![0; 6];
    let mut luma = LumaMut::new(&mut pixels, 3).unwrap();

    assert_eq!((luma.width(), luma.height()), (3, 2));
    for (row, values) in luma.rows_mut().enumerate() {
        values.fill(row as u8 + 1);
    }

    assert_eq!(pixels, [1, 1, 1, 2, 2, 2]);
}

#[test]
fn a_plane_that_is_not_a_whole_number_of_rows_is_refused() {
    for (length, width) in [(7, 3), (0, 3), (6, 0)] {
        let mut pixels = vec![0; length];

        let refused = LumaMut::new(&mut pixels, width).unwrap_err();

        assert_eq!(refused, Error { length, width });
    }
    assert_eq!(
        Error { length: 7, width: 3 }.to_string(),
        "a luma plane of 7 pixels is not a whole number of rows of 3 pixels"
    );
}
