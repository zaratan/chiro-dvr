use std::collections::BTreeSet;

use super::Script;

#[test]
fn every_text_of_the_script_has_its_own_key() {
    let keys: BTreeSet<&str> = Script::ALL.iter().map(|script| script.key()).collect();
    assert_eq!(keys.len(), Script::ALL.len());
}

#[test]
fn a_counted_text_names_the_holes_the_script_fills() {
    assert_eq!(Script::Counts.placeholders(), ["yes", "no", "doubt", "todo"]);
    assert_eq!(Script::Yes.placeholders(), [] as [&str; 0]);
}
