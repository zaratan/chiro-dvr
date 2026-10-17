use std::num::NonZeroUsize;
use std::panic;
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::thread;

#[cfg(test)]
mod tests;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Stop<S, C> {
    Source(S),
    Consumer(C),
}

type Delivery<Item, Slot, S> = Result<(Item, Slot), S>;

pub(crate) fn read_ahead<Slot, Item, S, C>(
    ahead: NonZeroUsize,
    make: impl FnMut() -> Slot + Send,
    fill: impl FnMut(&mut Slot) -> Result<Option<Item>, S> + Send,
    consume: impl FnMut(Item, &Slot) -> Result<(), C>,
) -> Result<(), Stop<S, C>>
where
    Slot: Send,
    Item: Send,
    S: Send,
{
    let slots = ahead.get().saturating_add(1);
    thread::scope(|scope| {
        let (ready_sender, ready) = mpsc::sync_channel::<Delivery<Item, Slot, S>>(slots);
        let (empty, empty_receiver) = mpsc::sync_channel::<Slot>(slots);
        let reader = scope.spawn(move || {
            let held = produce(slots, make, fill, &empty_receiver, &ready_sender);
            (empty_receiver, held)
        });
        let outcome = consume_all(&ready, &empty, consume);
        drop((ready, empty));
        reader.join().map_or_else(
            |cause| panic::resume_unwind(cause),
            |slots_kept_until_the_consumer_is_done| {
                drop(slots_kept_until_the_consumer_is_done);
                outcome
            },
        )
    })
}

fn produce<Slot, Item, S>(
    slots: usize,
    mut make: impl FnMut() -> Slot,
    mut fill: impl FnMut(&mut Slot) -> Result<Option<Item>, S>,
    empty: &Receiver<Slot>,
    ready: &SyncSender<Delivery<Item, Slot, S>>,
) -> Option<Slot> {
    let mut fresh = slots;
    loop {
        let mut slot = if fresh > 0 {
            fresh -= 1;
            make()
        } else {
            let Ok(slot) = empty.recv() else { return None };
            slot
        };
        let (delivery, held) = match fill(&mut slot) {
            Ok(Some(item)) => (Ok((item, slot)), None),
            Ok(None) => return Some(slot),
            Err(error) => (Err(error), Some(slot)),
        };
        let last = held.is_some();
        if ready.send(delivery).is_err() || last {
            return held;
        }
    }
}

fn consume_all<Slot, Item, S, C>(
    ready: &Receiver<Delivery<Item, Slot, S>>,
    empty: &SyncSender<Slot>,
    mut consume: impl FnMut(Item, &Slot) -> Result<(), C>,
) -> Result<(), Stop<S, C>> {
    for delivery in ready {
        let (item, slot) = delivery.map_err(Stop::Source)?;
        consume(item, &slot).map_err(Stop::Consumer)?;
        if empty.send(slot).is_err() {
            break;
        }
    }
    Ok(())
}
