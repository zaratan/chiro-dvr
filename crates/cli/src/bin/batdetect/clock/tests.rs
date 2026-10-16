use super::clock;

#[test]
fn a_length_names_only_the_units_it_needs_down_to_the_second() {
    assert_eq!(clock(0), "0 s");
    assert_eq!(clock(41), "41 s");
    assert_eq!(clock(301), "5 min 1 s");
    assert_eq!(clock(3600), "1 h 0 min 0 s");
    assert_eq!(clock(3725), "1 h 2 min 5 s");
}
