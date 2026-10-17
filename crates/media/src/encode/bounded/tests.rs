use std::panic;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use types::rational::Rational;

use super::Bounded;
use crate::encode::{Codec, Encoder, Error, Settings};
use crate::testing::{self, HEIGHT, WIDTH};

const SHORT: Duration = Duration::from_millis(100);
const GENEROUS: Duration = Duration::from_secs(10);

struct Flagged(Arc<AtomicBool>);

impl Drop for Flagged {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

fn never_answering(bounded: &Bounded) -> (Result<bool, Error>, Duration, mpsc::Sender<()>) {
    let (keep, wait) = mpsc::channel::<()>();
    let started = Instant::now();
    let result = bounded.start(move || Ok(wait.recv().is_ok()));
    (result, started.elapsed(), keep)
}

#[test]
fn a_start_that_answers_in_time_is_returned_and_the_service_stays_trusted() {
    let bounded = Bounded::new("answering", GENEROUS);
    assert_eq!(bounded.start(|| Ok(7)).unwrap(), 7);
    assert!(!bounded.is_silent());
}

#[test]
fn a_start_that_never_answers_is_given_up_at_the_limit_and_silences_the_service() {
    let bounded = Bounded::new("mute", SHORT);
    let (result, waited, _keep) = never_answering(&bounded);
    assert!(matches!(result, Err(Error::Unanswered { name: "mute", limit }) if limit == SHORT));
    assert!(waited >= SHORT && waited < GENEROUS, "{waited:?}");
    assert!(bounded.is_silent());
}

#[test]
fn once_silent_the_service_is_not_asked_again_for_the_rest_of_the_run_and_refuses_at_once() {
    let bounded = Bounded::new("mute", SHORT);
    let (_, _, _keep) = never_answering(&bounded);
    let asked = Arc::new(AtomicBool::new(false));
    let noted = Arc::clone(&asked);
    let started = Instant::now();
    let again = bounded.start(move || {
        noted.store(true, Ordering::SeqCst);
        Ok(())
    });
    assert!(matches!(again, Err(Error::Unanswered { name: "mute", .. })));
    assert!(started.elapsed() < SHORT, "{:?}", started.elapsed());
    assert!(!asked.load(Ordering::SeqCst));
}

#[test]
fn a_start_that_answers_after_the_limit_is_dropped_rather_than_left_open() {
    let bounded = Bounded::new("late", Duration::from_millis(20));
    let dropped = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&dropped);
    let result = bounded.start(move || {
        thread::sleep(Duration::from_millis(200));
        Ok(Flagged(flag))
    });
    assert!(result.is_err());
    let deadline = Instant::now() + GENEROUS;
    while !dropped.load(Ordering::SeqCst) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(10));
    }
    assert!(dropped.load(Ordering::SeqCst));
}

#[test]
fn a_start_that_fails_in_time_keeps_its_own_error_and_the_service_trusted() {
    let bounded = Bounded::new("refusing", GENEROUS);
    let result: Result<(), Error> = bounded.start(|| Err(Error::EncoderMissing { name: "refusing" }));
    assert!(matches!(result, Err(Error::EncoderMissing { name: "refusing" })));
    assert!(!bounded.is_silent());
}

#[test]
fn a_start_that_panics_passes_its_panic_on_instead_of_reading_as_a_silence() {
    let bounded = Bounded::new("panicking", GENEROUS);
    let caught = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        bounded.start::<()>(|| panic!("the start broke"))
    }));
    let payload = caught.unwrap_err();
    assert_eq!(payload.downcast_ref::<&str>(), Some(&"the start broke"));
    assert!(!bounded.is_silent());
}

#[test]
fn an_encoder_opened_on_the_bounding_thread_keeps_encoding_on_the_caller() {
    let folder = tempfile::tempdir().unwrap();
    let path = folder.path().join("bounded.mp4");
    let settings = Settings::new(WIDTH, HEIGHT, Rational::new(30, 1).unwrap(), Codec::OpenH264).unwrap();
    let target = path.clone();
    let bounded = Bounded::new("libopenh264", GENEROUS);
    let mut encoder = bounded.start(move || Encoder::create(&target, &settings)).unwrap();
    for index in 0..10 {
        encoder.push(&testing::pattern(index)).unwrap();
    }
    encoder.finish().unwrap();
    assert!(path.exists());
}
