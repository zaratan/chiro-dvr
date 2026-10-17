use std::path::PathBuf;

use media::encode::Codec;
use media::encode::fallback::Fallback;

use super::{Checkup, ENOUGH_FREE_BYTES, Encoding, Space, encoding, space};
use crate::access::Access;

fn ready() -> Checkup {
    Checkup {
        folder: PathBuf::from("/nuit"),
        encoding: Encoding::Preferred(Codec::OpenH264),
        space: Space::Enough(ENOUGH_FREE_BYTES),
        refused_writing: None,
    }
}

#[test]
fn an_encoder_that_passes_the_trial_first_is_the_preferred_one() {
    let only = Fallback::new(vec![Codec::OpenH264]).unwrap();
    assert_eq!(encoding(only), Encoding::Preferred(Codec::OpenH264));
}

#[cfg(not(target_os = "macos"))]
#[test]
fn videotoolbox_missing_falls_back_on_openh264_and_says_which_was_wanted() {
    let both = Fallback::new(vec![Codec::VideoToolbox, Codec::OpenH264]).unwrap();
    assert_eq!(encoding(both), Encoding::Fallback);
}

#[cfg(not(target_os = "macos"))]
#[test]
fn no_encoder_that_passes_the_trial_is_a_broken_installation() {
    let missing = Fallback::new(vec![Codec::VideoToolbox]).unwrap();
    assert_eq!(encoding(missing), Encoding::Broken);
}

#[test]
fn twenty_gigabytes_free_are_enough_and_one_byte_less_is_short() {
    assert_eq!(space(Some(ENOUGH_FREE_BYTES)), Space::Enough(ENOUGH_FREE_BYTES));
    assert_eq!(space(Some(ENOUGH_FREE_BYTES - 1)), Space::Short(ENOUGH_FREE_BYTES - 1));
}

#[test]
fn a_free_space_that_cannot_be_read_is_unknown_not_short() {
    assert_eq!(space(None), Space::Unknown);
}

#[test]
fn only_an_encoder_broken_or_not_tried_short_space_or_refused_writing_count_as_problems() {
    assert_eq!(ready().problems(), 0);
    let fallback = Checkup {
        encoding: Encoding::Fallback,
        space: Space::Unknown,
        ..ready()
    };
    assert_eq!(fallback.problems(), 0);
    let everything = Checkup {
        encoding: Encoding::Broken,
        space: Space::Short(1),
        refused_writing: Some(Access::ReadOnly),
        ..ready()
    };
    assert_eq!(everything.problems(), 3);
    let untried = Checkup {
        encoding: Encoding::Untried,
        ..ready()
    };
    assert_eq!(untried.problems(), 1);
}

#[cfg(target_os = "linux")]
#[test]
fn a_writable_folder_on_this_machine_passes_with_openh264() {
    use super::checkup;
    let folder = tempfile::tempdir().unwrap();
    let found = checkup(folder.path());
    assert_eq!(found.encoding, Encoding::Preferred(Codec::OpenH264));
    assert_eq!(found.refused_writing, None);
    assert_ne!(found.space, Space::Unknown);
}

#[cfg(unix)]
#[test]
fn a_folder_without_the_right_to_write_is_refused() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    use super::checkup;
    let folder = tempfile::tempdir().unwrap();
    fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let found = checkup(folder.path());
    fs::set_permissions(folder.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert_eq!(found.refused_writing, Some(Access::Refused));
}
