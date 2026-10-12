use std::env;
use std::ffi::OsString;

use types::language::Language;

#[cfg(test)]
mod tests;

const VARIABLES: [&str; 4] = ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"];
const FRENCH: &str = "fr";
const ENGLISH: &str = "en";

pub fn system_language() -> Language {
    language_of(|name| env::var_os(name))
}

fn language_of(variable: impl Fn(&str) -> Option<OsString>) -> Language {
    let chosen = VARIABLES
        .iter()
        .filter_map(|name| variable(name))
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    let chosen = chosen.to_string_lossy().to_ascii_lowercase();
    let known = chosen
        .split(':')
        .find(|item| item.starts_with(FRENCH) || item.starts_with(ENGLISH));
    if known.is_some_and(|item| item.starts_with(FRENCH)) {
        Language::French
    } else {
        Language::English
    }
}
