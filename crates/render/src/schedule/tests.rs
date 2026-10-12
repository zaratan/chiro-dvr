use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use super::{Hold, Lease, run};
use crate::clips::Error;

fn failure(item: usize) -> Error {
    Error::NoEncoder {
        path: PathBuf::from(item.to_string()),
    }
}

#[test]
fn the_items_held_at_once_never_need_more_sessions_than_there_are_and_results_keep_their_order() {
    let (live, most) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let items = [6, 6, 3, 6, 2, 8, 1];
    let results = run(
        &items,
        4,
        8,
        |need| *need,
        |(): &mut (), need, _| {
            let now = live.fetch_add(*need, Ordering::SeqCst) + need;
            most.fetch_max(now, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(15));
            live.fetch_sub(*need, Ordering::SeqCst);
            Ok(*need)
        },
    )
    .unwrap();
    assert_eq!(results, items);
    assert!(most.load(Ordering::SeqCst) <= 8);
    assert!(most.load(Ordering::SeqCst) > 6);
}

#[test]
fn items_without_sessions_run_side_by_side() {
    let (live, most) = (AtomicUsize::new(0), AtomicUsize::new(0));
    run(
        &[0; 6],
        3,
        8,
        |need| *need,
        |(): &mut (), _, _| {
            let now = live.fetch_add(1, Ordering::SeqCst) + 1;
            most.fetch_max(now, Ordering::SeqCst);
            thread::sleep(Duration::from_millis(30));
            live.fetch_sub(1, Ordering::SeqCst);
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(most.load(Ordering::SeqCst), 3);
}

#[test]
fn a_failure_leaves_the_waiting_items_unstarted_and_wins_over_the_stops_it_causes() {
    let started = Mutex::new(Vec::new());
    let items: Vec<usize> = (0..10).collect();
    let error = run(
        &items,
        2,
        8,
        |item| if *item == 1 { 0 } else { 8 },
        |(): &mut (), item, lease: &Lease<'_>| {
            started.lock().unwrap().push(*item);
            match item {
                0 => {
                    while !lease.stopped() {
                        thread::sleep(Duration::from_millis(1));
                    }
                    Err(Error::Stopped)
                }
                1 => Err(failure(1)),
                _ => Ok(()),
            }
        },
    )
    .unwrap_err();
    assert!(matches!(error, Error::NoEncoder { path } if path == Path::new("1")));
    let mut started = started.into_inner().unwrap();
    started.sort_unstable();
    assert_eq!(started, [0, 1]);
}

#[test]
fn the_error_of_the_first_failing_item_is_the_one_returned() {
    let items: Vec<usize> = (0..6).collect();
    let error = run(
        &items,
        1,
        8,
        |_| 0,
        |(): &mut (), item, _| if *item >= 2 { Err(failure(*item)) } else { Ok(()) },
    )
    .unwrap_err();
    assert!(matches!(error, Error::NoEncoder { path } if path == Path::new("2")));
}

#[test]
fn each_thread_keeps_its_own_state_from_one_item_to_the_next() {
    let seen = run(
        &[(); 5],
        1,
        8,
        |()| 0,
        |count: &mut usize, (), _| {
            *count += 1;
            Ok(*count)
        },
    )
    .unwrap();
    assert_eq!(seen, [1, 2, 3, 4, 5]);
}

#[test]
fn a_panic_reaches_the_caller_and_leaves_the_waiting_items_unstarted() {
    let started = Mutex::new(Vec::new());
    let items: Vec<usize> = (0..10).collect();
    let outcome = std::panic::catch_unwind(|| {
        run(
            &items,
            2,
            8,
            |_| 0,
            |(): &mut (), item: &usize, lease: &Lease<'_>| {
                started.lock().unwrap().push(*item);
                assert_ne!(*item, 1, "forced");
                let mut waited = 0;
                while *item == 0 && !lease.stopped() && waited < 2000 {
                    thread::sleep(Duration::from_millis(1));
                    waited += 1;
                }
                Ok(())
            },
        )
    });
    assert!(outcome.is_err());
    let mut started = started.into_inner().unwrap_or_else(std::sync::PoisonError::into_inner);
    started.sort_unstable();
    assert_eq!(started, [0, 1]);
}

#[test]
fn an_item_that_asks_for_more_sessions_than_there_are_takes_them_all_instead_of_waiting() {
    let results = run(&[20, 3], 2, 8, |need| *need, |(): &mut (), need, _| Ok(*need)).unwrap();
    assert_eq!(results, [20, 3]);
}

#[test]
fn sessions_given_back_on_the_way_let_the_next_item_start_before_the_first_ends() {
    let second_started = std::sync::atomic::AtomicBool::new(false);
    let overlapped = run(
        &[0, 1],
        2,
        8,
        |_| 8,
        |(): &mut (), item: &usize, lease: &Lease<'_>| {
            if *item == 1 {
                second_started.store(true, Ordering::SeqCst);
                return Ok(true);
            }
            lease.keep(0);
            let mut waited = 0;
            while !second_started.load(Ordering::SeqCst) && waited < 2000 {
                thread::sleep(Duration::from_millis(1));
                waited += 1;
            }
            Ok(second_started.load(Ordering::SeqCst))
        },
    )
    .unwrap();
    assert_eq!(overlapped, [true, true]);
}

#[test]
fn items_giving_their_sessions_back_step_by_step_never_hold_more_than_the_ceiling_and_all_finish() {
    let (live, most) = (AtomicUsize::new(0), AtomicUsize::new(0));
    let items: Vec<usize> = (0..40).map(|item| 1 + item * 5 % 8).collect();
    let results = run(
        &items,
        5,
        8,
        |need| *need,
        |(): &mut (), need: &usize, lease: &Lease<'_>| {
            let now = live.fetch_add(*need, Ordering::SeqCst) + need;
            most.fetch_max(now, Ordering::SeqCst);
            for kept in (0..*need).rev() {
                thread::sleep(Duration::from_millis(1));
                live.fetch_sub(1, Ordering::SeqCst);
                lease.keep(kept);
            }
            Ok(*need)
        },
    )
    .unwrap();
    assert_eq!(results, items);
    assert!(most.load(Ordering::SeqCst) <= 8);
}

#[test]
fn keeping_more_sessions_than_held_takes_none_more() {
    let both = std::sync::atomic::AtomicUsize::new(0);
    run(
        &[2, 6],
        2,
        8,
        |need| *need,
        |(): &mut (), _, lease: &Lease<'_>| {
            lease.keep(8);
            both.fetch_add(1, Ordering::SeqCst);
            let mut waited = 0;
            while both.load(Ordering::SeqCst) < 2 && waited < 2000 {
                thread::sleep(Duration::from_millis(1));
                waited += 1;
            }
            Ok(())
        },
    )
    .unwrap();
    assert_eq!(both.load(Ordering::SeqCst), 2);
}
