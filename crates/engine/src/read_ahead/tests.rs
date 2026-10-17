use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use super::{Stop, read_ahead};

const AHEAD: NonZeroUsize = NonZeroUsize::MIN.saturating_add(2);
const GUARD: Duration = Duration::from_secs(10);

#[derive(Debug, PartialEq, Eq)]
struct SourceFailed;

#[derive(Debug, PartialEq, Eq)]
struct ConsumerFailed;

fn finishes<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || sender.send(work()).unwrap());
    receiver.recv_timeout(GUARD).unwrap()
}

fn wait_until(condition: impl Fn() -> bool) {
    let deadline = Instant::now() + GUARD;
    while !condition() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(1));
    }
}

struct Tracked(Arc<AtomicUsize>);

impl Drop for Tracked {
    fn drop(&mut self) {
        self.0.fetch_sub(1, Ordering::SeqCst);
    }
}

struct Ending {
    next: usize,
    limit: usize,
    gone: Arc<AtomicBool>,
}

impl Ending {
    fn item(&mut self) -> Option<usize> {
        (self.next < self.limit).then(|| {
            self.next += 1;
            self.next - 1
        })
    }
}

impl Drop for Ending {
    fn drop(&mut self) {
        self.gone.store(true, Ordering::SeqCst);
    }
}

fn counting(limit: usize) -> impl FnMut(&mut usize) -> Result<Option<usize>, SourceFailed> + Send {
    let mut next = 0;
    move |slot| {
        if next == limit {
            return Ok(None);
        }
        *slot = next;
        next += 1;
        Ok(Some(next - 1))
    }
}

#[test]
fn items_arrive_in_order_in_slots_made_once_and_reused() {
    let (received, made) = finishes(|| {
        let made = AtomicUsize::new(0);
        let mut received = Vec::new();
        let new_slot = || {
            made.fetch_add(1, Ordering::SeqCst);
            0
        };
        let outcome = read_ahead(AHEAD, new_slot, counting(50), |item, slot: &usize| {
            received.push((item, *slot));
            Ok::<(), ConsumerFailed>(())
        });
        assert_eq!(outcome, Ok(()));
        (received, made.into_inner())
    });
    assert_eq!(received, (0..50).map(|item| (item, item)).collect::<Vec<_>>());
    assert_eq!(made, AHEAD.get() + 1);
}

#[test]
fn every_slot_made_lives_until_the_consumer_is_done_even_once_the_reader_has_met_the_end() {
    let (alive_at_the_last_item, alive_after) = finishes(|| {
        let alive = Arc::new(AtomicUsize::new(0));
        let gone = Arc::new(AtomicBool::new(false));
        let mut source = Ending {
            next: 0,
            limit: 2,
            gone: Arc::clone(&gone),
        };
        let counted = Arc::clone(&alive);
        let new_slot = move || {
            counted.fetch_add(1, Ordering::SeqCst);
            Tracked(Arc::clone(&counted))
        };
        let mut seen = None;
        let outcome = read_ahead(
            AHEAD,
            new_slot,
            move |_: &mut Tracked| Ok::<_, SourceFailed>(source.item()),
            |item, _: &Tracked| {
                if item == 0 {
                    wait_until(|| gone.load(Ordering::SeqCst));
                } else {
                    seen = Some(alive.load(Ordering::SeqCst));
                }
                Ok::<(), ConsumerFailed>(())
            },
        );
        assert_eq!(outcome, Ok(()));
        (seen, alive.load(Ordering::SeqCst))
    });
    assert_eq!(alive_at_the_last_item, Some(3));
    assert_eq!(alive_after, 0);
}

#[test]
fn an_empty_source_gives_nothing() {
    let received = finishes(|| {
        let mut received = Vec::new();
        let outcome = read_ahead(
            AHEAD,
            || 0,
            counting(0),
            |item, _: &usize| {
                received.push(item);
                Ok::<(), ConsumerFailed>(())
            },
        );
        assert_eq!(outcome, Ok(()));
        received
    });
    assert_eq!(received, Vec::<usize>::new());
}

#[test]
fn a_source_error_comes_after_the_items_already_read_with_its_own_type() {
    let (received, outcome) = finishes(|| {
        let mut received = Vec::new();
        let mut items = counting(2);
        let failing = move |slot: &mut usize| items(slot).and_then(|item| item.map(Some).ok_or(SourceFailed));
        let outcome = read_ahead(
            AHEAD,
            || 0,
            failing,
            |item, _: &usize| {
                received.push(item);
                Ok::<(), ConsumerFailed>(())
            },
        );
        (received, outcome)
    });
    assert_eq!(received, [0, 1]);
    assert_eq!(outcome, Err(Stop::Source(SourceFailed)));
}

#[test]
fn a_consumer_error_ends_an_endless_reader_and_wins() {
    let outcome = finishes(|| {
        read_ahead(
            AHEAD,
            || 0,
            counting(usize::MAX),
            |item, _: &usize| {
                if item == 3 { Err(ConsumerFailed) } else { Ok(()) }
            },
        )
    });
    assert_eq!(outcome, Err(Stop::Consumer(ConsumerFailed)));
}

#[test]
fn the_reader_stays_exactly_ahead_items_in_front_of_a_waiting_consumer() {
    let filled = finishes(|| {
        let filled = AtomicUsize::new(0);
        let mut items = counting(usize::MAX);
        let source = |slot: &mut usize| {
            filled.fetch_add(1, Ordering::SeqCst);
            items(slot)
        };
        let mut seen = None;
        let outcome = read_ahead(
            AHEAD,
            || 0,
            source,
            |_, _: &usize| {
                wait_until(|| filled.load(Ordering::SeqCst) > AHEAD.get());
                thread::sleep(Duration::from_millis(50));
                seen = Some(filled.load(Ordering::SeqCst));
                Err(ConsumerFailed)
            },
        );
        assert_eq!(outcome, Err(Stop::Consumer(ConsumerFailed)));
        seen
    });
    assert_eq!(filled, Some(AHEAD.get() + 1));
}

#[test]
fn a_panicking_reader_hands_its_own_cause_to_the_caller() {
    let reading = thread::spawn(|| {
        read_ahead(
            AHEAD,
            || 0,
            |_: &mut usize| -> Result<Option<usize>, SourceFailed> { panic!("broken source") },
            |_, _: &usize| Ok::<(), ConsumerFailed>(()),
        )
    });
    let cause = reading.join().unwrap_err();
    assert_eq!(cause.downcast_ref::<&str>(), Some(&"broken source"));
}

#[test]
#[should_panic(expected = "consumer panic")]
fn a_panic_of_the_consumer_reaches_the_caller_without_leaving_the_reader_blocked() {
    let outcome = read_ahead(
        AHEAD,
        || 0,
        counting(1_000),
        |item, _: &usize| {
            assert_ne!(item, 3, "consumer panic");
            Ok::<(), ConsumerFailed>(())
        },
    );
    drop(outcome);
}
