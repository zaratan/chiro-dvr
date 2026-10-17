use super::{Build, CHANGED, CLEAN, Refusal};

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
fn a_prerelease_tag_names_its_own_reference_rather_than_the_release_it_comes_before() {
    let releases: Vec<String> = ["v0.3.0-rc1", "v0.3.0-rc1-4-g28d5526", "v0.3.0-4-g28d5526"]
        .map(|described| Build::clean(described, COMMIT, RUSTC).unwrap().release().to_owned())
        .to_vec();

    assert_eq!(releases, ["v0.3.0-rc1", "v0.3.0-rc1", "v0.3.0"]);
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

#[test]
fn a_change_or_an_untracked_file_listed_by_git_status_makes_the_build_dirty_once() {
    let dirty = |described: &str| Refusal::Dirty {
        described: described.to_owned(),
    };

    assert_eq!(
        Build::engraved("v0.2.0-3-g28d5526", CHANGED, COMMIT, RUSTC),
        Err(dirty("v0.2.0-3-g28d5526-dirty"))
    );
    assert_eq!(
        Build::engraved("v0.2.0-3-g28d5526-dirty", CHANGED, COMMIT, RUSTC),
        Err(dirty("v0.2.0-3-g28d5526-dirty"))
    );
    assert_eq!(
        Build::engraved("v0.2.0-3-g28d5526", CLEAN, COMMIT, RUSTC),
        Build::clean("v0.2.0-3-g28d5526", COMMIT, RUSTC)
    );
}

#[test]
fn a_git_status_that_could_not_run_leaves_the_provenance_unknown_rather_than_clean() {
    assert_eq!(
        Build::engraved("v0.2.0-3-g28d5526", "", COMMIT, RUSTC),
        Err(Refusal::Unknown)
    );
    assert_eq!(Build::engraved("", CHANGED, COMMIT, RUSTC), Err(Refusal::Unknown));
}

#[test]
fn the_status_engraved_by_the_build_script_is_one_the_provenance_reads() {
    assert!([CLEAN, CHANGED, ""].contains(&env!("RECORD_STATUS")));
}
