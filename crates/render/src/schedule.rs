use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex, MutexGuard, PoisonError};
use std::thread;

use crate::clips::Error;

#[cfg(test)]
mod tests;

#[derive(Debug)]
struct Board {
    next: usize,
    free: usize,
    stopped: bool,
}

#[derive(Debug)]
struct Shared {
    board: Mutex<Board>,
    changed: Condvar,
    stop: AtomicBool,
}

#[derive(Debug)]
struct Taken<'a> {
    shared: &'a Shared,
    index: usize,
    sessions: usize,
}

impl Drop for Taken<'_> {
    fn drop(&mut self) {
        self.shared.board().free += self.sessions;
        self.shared.changed.notify_all();
    }
}

impl Shared {
    fn board(&self) -> MutexGuard<'_, Board> {
        self.board.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn take<G>(&self, items: &[G], sessions: &impl Fn(&G) -> usize) -> Option<Taken<'_>> {
        let needed = |board: &Board| items.get(board.next).map(sessions);
        let board = self.board();
        let mut board = self
            .changed
            .wait_while(board, |board| {
                !board.stopped && needed(board).is_some_and(|wanted| wanted > board.free)
            })
            .unwrap_or_else(PoisonError::into_inner);
        let wanted = needed(&board).filter(|_| !board.stopped)?;
        let index = board.next;
        board.next += 1;
        board.free -= wanted;
        Some(Taken {
            shared: self,
            index,
            sessions: wanted,
        })
    }

    fn halt(&self) {
        self.stop.store(true, Ordering::Release);
        self.board().stopped = true;
        self.changed.notify_all();
    }
}

pub(crate) fn run<G, S, T>(
    items: &[G],
    threads: usize,
    sessions: usize,
    needed: impl Fn(&G) -> usize + Sync,
    work: impl Fn(&mut S, &G, &AtomicBool) -> Result<T, Error> + Sync,
) -> Result<Vec<T>, Error>
where
    G: Sync,
    S: Default,
    T: Send,
{
    let shared = Shared {
        board: Mutex::new(Board {
            next: 0,
            free: sessions,
            stopped: false,
        }),
        changed: Condvar::new(),
        stop: AtomicBool::new(false),
    };
    let workers = threads.clamp(1, items.len().max(1));
    let mut done: Vec<(usize, Result<T, Error>)> = thread::scope(|scope| {
        let running: Vec<_> = (0..workers)
            .map(|_| scope.spawn(|| worker(&shared, items, &needed, &work)))
            .collect();
        running
            .into_iter()
            .flat_map(|worker| worker.join().unwrap_or_else(|cause| panic::resume_unwind(cause)))
            .collect()
    });
    done.sort_by_key(|(index, _)| *index);
    let (mut results, mut stopped, mut failed) = (Vec::with_capacity(done.len()), None, None);
    for (_, result) in done {
        match result {
            Ok(value) => results.push(value),
            Err(Error::Stopped) => stopped = Some(Error::Stopped),
            Err(error) => {
                failed.get_or_insert(error);
            }
        }
    }
    match failed.or(stopped) {
        Some(error) => Err(error),
        None => Ok(results),
    }
}

fn worker<G, S, T>(
    shared: &Shared,
    items: &[G],
    needed: &impl Fn(&G) -> usize,
    work: &impl Fn(&mut S, &G, &AtomicBool) -> Result<T, Error>,
) -> Vec<(usize, Result<T, Error>)>
where
    S: Default,
{
    let mut state = S::default();
    let mut done = Vec::new();
    while let Some(taken) = shared.take(items, needed) {
        let Some(item) = items.get(taken.index) else {
            break;
        };
        let result = work(&mut state, item, &shared.stop);
        if result.is_err() {
            shared.halt();
        }
        done.push((taken.index, result));
    }
    done
}
