use media::encode::Codec;
use media::encode::fallback::Fallback;

#[cfg(test)]
#[path = "support/round_trip.rs"]
mod round_trip;

const OPENH264_LUMA_PSNR: f64 = 39.1;

#[test]
#[ignore = "slow"]
fn a_passage_of_the_extract_encoded_by_openh264_at_qp_23_reads_back_whole_at_the_source_rate() {
    let fallback = Fallback::new(vec![Codec::OpenH264]).unwrap();
    let (stream, luma_psnr) = round_trip::round_trip(&fallback);
    assert_eq!(stream, round_trip::expected(Codec::OpenH264));
    assert!(luma_psnr > OPENH264_LUMA_PSNR, "luma PSNR {luma_psnr}");
}
