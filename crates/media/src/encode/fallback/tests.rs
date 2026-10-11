use types::rational::Rational;

use super::{Error, Fallback};
use crate::encode::{self, Codec, Encoder, Settings};
use crate::testing::{self, HEIGHT, WIDTH};

const HARDWARE: Codec = Codec::VideoToolbox;
const WIDER_THAN_THE_HARDWARE: (u32, u32) = (4112, 1088);
const BEYOND_BOTH: (u32, u32) = (4112, 2304);

fn thirty() -> Rational {
    Rational::new(30, 1).unwrap()
}

fn hardware_refuses(width: u32, height: u32) -> bool {
    let folder = tempfile::tempdir().unwrap();
    let settings = Settings::new(width, height, thirty(), HARDWARE).unwrap();
    matches!(
        Encoder::create(&folder.path().join("probe.mp4"), &settings),
        Err(encode::Error::Open { .. } | encode::Error::EncoderMissing { .. })
    )
}

#[test]
fn an_empty_list_is_refused() {
    assert!(matches!(Fallback::new(Vec::new()), Err(Error::Empty)));
}

#[test]
fn a_codec_listed_twice_is_refused() {
    assert!(matches!(
        Fallback::new(vec![Codec::OpenH264, HARDWARE, Codec::OpenH264]),
        Err(Error::Repeated(Codec::OpenH264))
    ));
}

#[test]
fn the_first_encoder_that_opens_writes_the_clip() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("clip.mp4");
    let fallback = Fallback::new(vec![Codec::OpenH264, HARDWARE]).unwrap();
    let mut encoder = fallback.create(&path, WIDTH, HEIGHT, thirty()).unwrap();
    assert_eq!(encoder.codec(), Codec::OpenH264);
    encoder.push(&testing::pattern(0)).unwrap();
    encoder.finish().unwrap();
    assert!(path.exists());
}

#[test]
fn a_videotoolbox_that_refuses_the_clip_falls_back_to_openh264() {
    let (width, height) = WIDER_THAN_THE_HARDWARE;
    assert!(
        hardware_refuses(width, height),
        "the hardware now takes {width}×{height}: pick a wider size"
    );
    let folder = tempfile::tempdir().unwrap();
    let fallback = Fallback::new(vec![HARDWARE, Codec::OpenH264]).unwrap();
    let encoder = fallback
        .create(&folder.path().join("clip.mp4"), width, height, thirty())
        .unwrap();
    assert_eq!(encoder.codec(), Codec::OpenH264);
}

#[test]
fn a_trial_drops_the_leading_encoders_that_fail() {
    let (width, height) = WIDER_THAN_THE_HARDWARE;
    assert!(
        hardware_refuses(width, height),
        "the hardware now takes {width}×{height}: pick a wider size"
    );
    let fallback = Fallback::new(vec![HARDWARE, Codec::OpenH264]).unwrap();
    let tried = fallback.trial(width, height, thirty()).unwrap();
    assert_eq!(tried.codecs().collect::<Vec<_>>(), [Codec::OpenH264]);
}

#[test]
fn a_trial_keeps_the_encoders_after_the_first_that_works() {
    let fallback = Fallback::new(vec![Codec::OpenH264, HARDWARE]).unwrap();
    let tried = fallback.clone().trial(WIDTH, HEIGHT, thirty()).unwrap();
    assert_eq!(tried, fallback);
}

#[test]
fn when_no_encoder_opens_every_failure_is_reported() {
    let (width, height) = BEYOND_BOTH;
    let folder = tempfile::tempdir().unwrap();
    let fallback = Fallback::new(vec![HARDWARE, Codec::OpenH264]).unwrap();
    let error = fallback
        .create(&folder.path().join("huge.mp4"), width, height, thirty())
        .unwrap_err();
    let Error::NoEncoder { failures } = &error else {
        panic!("{error}");
    };
    let codecs: Vec<Codec> = failures.iter().map(|(codec, _)| *codec).collect();
    assert_eq!(codecs, [HARDWARE, Codec::OpenH264]);
    let message = error.to_string();
    assert!(message.contains("VideoToolbox hardware: "), "{message}");
    assert!(
        message.contains("; openh264: cannot start the libopenh264 encoder: "),
        "{message}"
    );
    assert_eq!(std::fs::read_dir(folder.path()).unwrap().count(), 0);
}

#[test]
fn a_trial_where_no_encoder_works_reports_them_all() {
    let (width, height) = BEYOND_BOTH;
    let fallback = Fallback::new(vec![HARDWARE, Codec::OpenH264]).unwrap();
    assert!(matches!(
        fallback.trial(width, height, thirty()),
        Err(Error::NoEncoder { failures }) if failures.len() == 2
    ));
}

#[test]
fn an_odd_size_is_refused_once_for_the_whole_list() {
    let folder = tempfile::tempdir().unwrap();
    let fallback = Fallback::for_this_platform();
    assert!(matches!(
        fallback.create(&folder.path().join("clip.mp4"), 321, 240, thirty()),
        Err(Error::Settings(encode::Error::Size { .. }))
    ));
}

#[test]
fn a_trial_frame_too_large_for_memory_is_refused_before_any_encoder() {
    let side = u32::MAX - 1;
    assert!(matches!(
        Fallback::for_this_platform().trial(side, side, thirty()),
        Err(Error::Frame(_))
    ));
}

#[test]
fn this_platform_ends_with_openh264_after_the_hardware_on_macos() {
    let expected: &[Codec] = if cfg!(target_os = "macos") {
        &[HARDWARE, Codec::OpenH264]
    } else {
        &[Codec::OpenH264]
    };
    assert_eq!(Fallback::for_this_platform().codecs().collect::<Vec<_>>(), expected);
}

#[test]
fn a_missing_folder_is_a_destination_error_rather_than_a_failure_of_every_encoder() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("absent/clip.mp4");
    let fallback = Fallback::new(vec![HARDWARE, Codec::OpenH264]).unwrap();
    assert!(matches!(
        fallback.create(&path, WIDTH, HEIGHT, thirty()),
        Err(Error::Destination(encode::Error::Create { .. }))
    ));
}
