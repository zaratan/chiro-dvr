use std::panic;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use super::Error;

#[cfg(test)]
mod tests;

pub const VIDEOTOOLBOX_START_LIMIT: Duration = Duration::from_secs(10);

pub static VIDEOTOOLBOX: Bounded = Bounded::new("h264_videotoolbox", VIDEOTOOLBOX_START_LIMIT);

#[derive(Debug)]
pub struct Bounded {
    name: &'static str,
    limit: Duration,
    silent: AtomicBool,
}

impl Bounded {
    pub const fn new(name: &'static str, limit: Duration) -> Self {
        Self {
            name,
            limit,
            silent: AtomicBool::new(false),
        }
    }

    pub fn is_silent(&self) -> bool {
        self.silent.load(Ordering::Acquire)
    }

    pub fn start<T: Send + 'static>(
        &self,
        start: impl FnOnce() -> Result<T, Error> + Send + 'static,
    ) -> Result<T, Error> {
        let unanswered = Error::Unanswered {
            name: self.name,
            limit: self.limit,
        };
        if self.is_silent() {
            return Err(unanswered);
        }
        let (sender, receiver) = mpsc::sync_channel(1);
        let worker = thread::spawn(move || sender.send(start()).map_err(|late| drop(late.0)));
        match receiver.recv_timeout(self.limit) {
            Ok(answer) => answer,
            Err(RecvTimeoutError::Timeout) => {
                self.silent.store(true, Ordering::Release);
                Err(unanswered)
            }
            Err(RecvTimeoutError::Disconnected) => worker
                .join()
                .map_or_else(|payload| panic::resume_unwind(payload), |_| Err(unanswered)),
        }
    }
}
