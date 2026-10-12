use std::panic;
use std::sync::mpsc::{self, Receiver, SendError, SyncSender};
use std::thread;

use media::frame::{self, ColorRange, YuvFrame};
use types::frame::FrameIndex;
use types::span::Span;

use crate::clips::Error;
use crate::source::{self, Origin, Source};

#[cfg(test)]
mod tests;

pub(crate) const QUEUE_DEPTH: usize = 4;

#[derive(Debug)]
enum Halt {
    ConsumerGone,
    Source(source::Error),
    Frame(frame::Error),
}

impl From<source::Error> for Halt {
    fn from(error: source::Error) -> Self {
        Self::Source(error)
    }
}

pub(crate) fn serve_ahead(
    source: &Source,
    span: Span,
    consume: impl FnMut(FrameIndex, &YuvFrame) -> Result<(), Error>,
) -> Result<Origin, Error> {
    let slots = QUEUE_DEPTH + 1;
    thread::scope(|scope| {
        let (ready_sender, ready) = mpsc::sync_channel::<(FrameIndex, YuvFrame)>(QUEUE_DEPTH);
        let (empty, empty_receiver) = mpsc::sync_channel::<YuvFrame>(slots);
        let reader = scope.spawn(move || produce(source, span, slots, &empty_receiver, &ready_sender));
        let expected = span.frame_count();
        let (served, consumed) = consume_all(&ready, &empty, consume);
        drop((ready, empty));
        let produced = reader.join().unwrap_or_else(|cause| panic::resume_unwind(cause));
        consumed?;
        let origin = match produced {
            Ok(origin) => Some(origin),
            Err(Halt::Source(error)) => return Err(Error::Source(error)),
            Err(Halt::Frame(error)) => return Err(Error::Frame(error)),
            Err(Halt::ConsumerGone) => None,
        };
        match origin {
            Some(origin) if served == expected => Ok(origin),
            _ => Err(Error::Unserved { served, expected }),
        }
    })
}

fn produce(
    source: &Source,
    span: Span,
    slots: usize,
    empty: &Receiver<YuvFrame>,
    ready: &SyncSender<(FrameIndex, YuvFrame)>,
) -> Result<Origin, Halt> {
    let mut fresh = slots;
    source.serve(span, |index, frame| {
        let mut slot = if fresh > 0 {
            fresh -= 1;
            YuvFrame::new(frame.width(), frame.height(), ColorRange::Unspecified).map_err(Halt::Frame)?
        } else {
            empty.recv().map_err(|mpsc::RecvError| Halt::ConsumerGone)?
        };
        slot.copy_from(frame).map_err(Halt::Frame)?;
        ready.send((index, slot)).map_err(|SendError(_slot)| Halt::ConsumerGone)
    })
}

fn consume_all(
    ready: &Receiver<(FrameIndex, YuvFrame)>,
    empty: &SyncSender<YuvFrame>,
    mut consume: impl FnMut(FrameIndex, &YuvFrame) -> Result<(), Error>,
) -> (usize, Result<(), Error>) {
    let mut served = 0;
    for (index, slot) in ready {
        if let Err(error) = consume(index, &slot) {
            return (served, Err(error));
        }
        served += 1;
        if let Err(SendError(slot_of_a_finished_reader)) = empty.send(slot) {
            drop(slot_of_a_finished_reader);
        }
    }
    (served, Ok(()))
}
