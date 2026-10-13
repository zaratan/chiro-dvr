use super::median;

#[test]
fn the_median_of_an_odd_count_is_its_middle_value_and_of_an_even_count_the_mean_of_its_two_middle_values() {
    assert_eq!(median([3.0, -1.0, 2.0]), Some(2.0));
    assert_eq!(median([4.0, -1.0, 2.0, 10.0]), Some(3.0));
    assert_eq!(median([-12.5]), Some(-12.5));
}

#[test]
fn no_value_has_no_median() {
    assert_eq!(median([]), None);
}
