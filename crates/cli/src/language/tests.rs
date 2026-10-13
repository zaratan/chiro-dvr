use std::collections::HashMap;
use std::ffi::OsString;

use types::language::Language;

use super::language_of;

fn language(variables: &[(&str, &str)]) -> Language {
    language_with_system(variables, None)
}

fn language_with_system(variables: &[(&str, &str)], system: Option<&str>) -> Language {
    let variables: HashMap<String, OsString> = variables
        .iter()
        .map(|&(name, value)| (name.to_owned(), OsString::from(value)))
        .collect();
    language_of(|name| variables.get(name).cloned(), || system.map(str::to_owned))
}

#[test]
fn a_french_locale_gives_french() {
    assert_eq!(language(&[("LANG", "fr_FR.UTF-8")]), Language::French);
}

#[test]
fn no_locale_at_all_gives_english() {
    assert_eq!(language(&[]), Language::English);
    assert_eq!(language(&[("LANG", "C")]), Language::English);
}

#[test]
fn language_wins_over_lc_all_which_wins_over_lang() {
    assert_eq!(
        language(&[("LANGUAGE", "en"), ("LANG", "fr_FR.UTF-8")]),
        Language::English
    );
    assert_eq!(
        language(&[("LC_ALL", "fr_CA.UTF-8"), ("LANG", "en_GB.UTF-8")]),
        Language::French
    );
    assert_eq!(
        language(&[("LC_MESSAGES", "fr_BE"), ("LANG", "en_US")]),
        Language::French
    );
}

#[test]
fn an_empty_variable_is_skipped() {
    assert_eq!(language(&[("LANGUAGE", ""), ("LANG", "fr_FR.UTF-8")]), Language::French);
}

#[test]
fn the_first_language_of_a_list_that_has_messages_counts() {
    assert_eq!(language(&[("LANGUAGE", "fr:en")]), Language::French);
    assert_eq!(language(&[("LANGUAGE", "en:fr")]), Language::English);
    assert_eq!(language(&[("LANGUAGE", "de:fr:en")]), Language::French);
    assert_eq!(language(&[("LANGUAGE", "de")]), Language::English);
}

#[test]
fn without_any_variable_the_language_of_the_system_decides() {
    assert_eq!(language_with_system(&[], Some("fr-FR")), Language::French);
    assert_eq!(language_with_system(&[], Some("en-GB")), Language::English);
    assert_eq!(language_with_system(&[("LANG", "")], Some("fr-CA")), Language::French);
}

#[test]
fn a_variable_that_is_set_wins_over_the_language_of_the_system() {
    assert_eq!(language_with_system(&[("LANG", "C")], Some("fr-FR")), Language::English);
    assert_eq!(
        language_with_system(&[("LANG", "fr_FR.UTF-8")], Some("en-US")),
        Language::French
    );
}
