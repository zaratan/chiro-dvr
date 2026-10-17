use super::{Build, Refusal};

const COMMIT: &str = "28d5526d0b2c0f1d7e3a9b4c5d6e7f8091a2b3c4";
const RUSTC: &str = "rustc 1.90.0 (1159e78c4 2025-09-14)";

#[test]
fn a_binary_built_from_a_commit_without_changes_keeps_its_version_and_names_the_reference_by_its_release() {
    let build = Build::clean("v0.2.0-3-g28d5526", COMMIT, RUSTC).unwrap();

    assert_eq!(
        (build.version(), build.release(), build.commit(), build.rustc()),
        ("0.2.0-3-g28d5526", "v0.2.0", COMMIT, RUSTC)
    );
    assert_eq!(Build::clean("v0.3.0", COMMIT, RUSTC).unwrap().release(), "v0.3.0");
}

#[test]
fn a_binary_built_with_uncommitted_changes_records_nothing_and_says_from_what() {
    let refused = Build::clean("v0.2.0-3-g28d5526-dirty", COMMIT, RUSTC).unwrap_err();

    assert_eq!(
        refused,
        Refusal::Dirty {
            described: "v0.2.0-3-g28d5526-dirty".to_owned()
        }
    );
    assert!(refused.to_string().contains("v0.2.0-3-g28d5526-dirty"));
}

#[test]
fn a_binary_whose_commit_compiler_or_release_is_unknown_records_nothing() {
    assert_eq!(Build::clean("", "", RUSTC), Err(Refusal::Unknown));
    assert_eq!(Build::clean("v0.2.0", COMMIT, ""), Err(Refusal::Unknown));
    assert_eq!(
        Build::clean("28d5526", COMMIT, RUSTC),
        Err(Refusal::Untagged {
            described: "28d5526".to_owned()
        })
    );
}
