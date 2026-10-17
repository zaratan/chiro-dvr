use super::columns;

#[test]
fn a_file_has_no_width_so_nothing_is_cut() {
    assert_eq!(columns(tempfile::tempfile().unwrap()), None);
}
