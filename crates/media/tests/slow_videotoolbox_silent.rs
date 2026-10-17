use std::sync::mpsc;
use std::time::{Duration, Instant};

use media::encode::bounded::{VIDEOTOOLBOX, VIDEOTOOLBOX_START_LIMIT};
use media::encode::fallback::Fallback;
use media::encode::{Codec, Encoder, Error, Settings};
use types::rational::Rational;

const AT_ONCE: Duration = Duration::from_secs(1);

#[test]
#[ignore = "slow"]
fn once_videotoolbox_stays_silent_every_later_session_and_trial_of_the_run_goes_to_openh264_at_once() {
    let (_keep, never) = mpsc::channel::<()>();
    let started = Instant::now();
    let silenced = VIDEOTOOLBOX.start(move || Ok(never.recv().is_ok()));
    assert!(matches!(silenced, Err(Error::Unanswered { .. })));
    assert!(started.elapsed() >= VIDEOTOOLBOX_START_LIMIT);
    assert!(VIDEOTOOLBOX.is_silent());

    let folder = tempfile::tempdir().unwrap();
    let rate = Rational::new(30, 1).unwrap();
    let settings = Settings::new(320, 240, rate, Codec::VideoToolbox).unwrap();
    let asked = Instant::now();
    let session = Encoder::create(&folder.path().join("clip.mp4"), &settings);
    assert!(matches!(
        session,
        Err(Error::Unanswered {
            name: "h264_videotoolbox",
            ..
        })
    ));
    let trial = Fallback::new(vec![Codec::VideoToolbox, Codec::OpenH264])
        .unwrap()
        .trial(320, 240, rate)
        .unwrap();
    assert!(asked.elapsed() < AT_ONCE, "{:?}", asked.elapsed());
    assert_eq!(trial.codecs().collect::<Vec<_>>(), [Codec::OpenH264]);
}
