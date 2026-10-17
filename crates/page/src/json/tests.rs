use super::{object, string};

const BACKSLASH: char = '\\';

#[test]
fn a_string_cannot_close_the_script_that_holds_it() {
    assert_eq!(
        string("</script>"),
        format!("\"{BACKSLASH}u003c/script{BACKSLASH}u003e\"")
    );
}

#[test]
fn quotes_backslashes_and_control_characters_are_escaped() {
    assert_eq!(
        string("a\"b\\c\nd\u{2028}"),
        format!("\"a{BACKSLASH}\"b{BACKSLASH}{BACKSLASH}c{BACKSLASH}u000ad{BACKSLASH}u2028\"")
    );
}

#[test]
fn an_object_keeps_its_keys_in_order() {
    assert_eq!(
        object([("yes", "oui".to_owned()), ("no", "non".to_owned())]),
        r#"{"yes":"oui","no":"non"}"#
    );
}
