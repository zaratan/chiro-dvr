use super::DecodeErrors;

#[test]
fn no_bit_is_no_error() {
    let clean = DecodeErrors::from_bits(0);
    assert!(clean.is_empty());
    assert_eq!(clean, DecodeErrors::default());
}

#[test]
fn a_concealed_frame_with_missing_slices_keeps_both_meanings_and_its_raw_bits() {
    let concealed = DecodeErrors::from_bits(12);
    assert!(!concealed.is_empty());
    assert!(concealed.concealment_active() && concealed.slices_missing());
    assert!(!concealed.invalid_bitstream() && !concealed.missing_reference());
    assert_eq!(concealed.bits(), 12);
}

#[test]
fn each_ffmpeg_flag_has_its_own_name() {
    assert!(DecodeErrors::from_bits(1).invalid_bitstream());
    assert!(DecodeErrors::from_bits(2).missing_reference());
    assert!(DecodeErrors::from_bits(4).concealment_active());
    assert!(DecodeErrors::from_bits(8).slices_missing());
}

#[test]
fn a_bit_unknown_to_this_version_still_marks_an_error() {
    let unknown = DecodeErrors::from_bits(64);
    assert!(!unknown.is_empty());
    assert_eq!(unknown.bits(), 64);
}
