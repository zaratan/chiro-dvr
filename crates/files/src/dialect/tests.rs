use types::language::Language;

use super::Dialect;

fn cells(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|&text| text.to_owned()).collect()
}

#[test]
fn a_french_table_starts_with_a_byte_order_mark_so_a_spreadsheet_reads_its_accents() {
    assert_eq!(
        Dialect::of(Language::French).header(cells(&["piste", "début"])),
        "\u{feff}piste;début"
    );
    assert_eq!(
        Dialect::of(Language::English).header(cells(&["track", "start"])),
        "track,start"
    );
}

#[test]
fn a_french_number_takes_a_decimal_comma_and_an_english_one_a_point() {
    assert_eq!(Dialect::of(Language::French).decimal(9.62, 2), "9,62");
    assert_eq!(Dialect::of(Language::French).decimal(4.8, 2), "4,8");
    assert_eq!(Dialect::of(Language::English).decimal(9.62, 2), "9.62");
}

#[test]
fn a_whole_number_keeps_its_decimal_mark() {
    let english = Dialect::of(Language::English);
    assert_eq!(english.decimal(12.0, 1), "12.0");
    assert_eq!(english.decimal(7.3, 2), "7.3");
    assert_eq!(english.decimal(944.149, 1), "944.1");
    assert_eq!(english.decimal(0.000_4, 2), "0.0");
    assert_eq!(Dialect::of(Language::French).decimal(12.0, 1), "12,0");
}

#[test]
fn a_negative_zero_is_written_as_zero() {
    let english = Dialect::of(Language::English);
    assert_eq!(english.decimal(-0.0, 1), "0.0");
    assert_eq!(english.decimal(-0.01, 1), "0.0");
}

#[test]
fn a_cell_holding_the_separator_or_a_quote_is_quoted_so_the_columns_stay_in_place() {
    assert_eq!(
        Dialect::of(Language::English).row(cells(&["speed, mean", "say \"hi\"", "plain"])),
        "\"speed, mean\",\"say \"\"hi\"\"\",plain"
    );
    assert_eq!(Dialect::of(Language::French).row(cells(&["a;b", "1,5"])), "\"a;b\";1,5");
}
