#![cfg(target_os = "macos")]

use media::encode::Codec;
use media::encode::fallback::Fallback;

#[cfg(test)]
#[path = "support/round_trip.rs"]
mod round_trip;

const VIDEOTOOLBOX_LUMA_PSNR: f64 = 37.9;

#[test]
#[ignore = "hardware"]
fn a_passage_of_the_extract_encoded_by_the_videotoolbox_hardware_at_q65_reads_back_whole_at_the_source_rate() {
    let hardware = Codec::VideoToolbox;
    let (stream, luma_psnr) = round_trip::round_trip(&Fallback::new(vec![hardware]).unwrap());
    assert_eq!(stream, round_trip::expected(hardware));
    assert!(luma_psnr > VIDEOTOOLBOX_LUMA_PSNR, "luma PSNR {luma_psnr}");
}
