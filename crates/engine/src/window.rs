use std::collections::VecDeque;
use std::num::NonZeroUsize;

#[cfg(test)]
mod tests;

pub(crate) type Unit = u16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Prepared {
    pub(crate) units: Vec<Unit>,
    pub(crate) sum: u64,
}

#[derive(Debug)]
pub(crate) struct Window {
    half: usize,
    step: NonZeroUsize,
    pixels: usize,
    first: usize,
    frames: VecDeque<Prepared>,
    spare: Vec<Prepared>,
}

pub(crate) fn most_sampled(half: NonZeroUsize, step: NonZeroUsize) -> NonZeroUsize {
    NonZeroUsize::MIN.saturating_add(half.get().saturating_mul(2) / step.get())
}

impl Window {
    pub(crate) fn new(half: NonZeroUsize, step: NonZeroUsize, pixels: usize) -> Self {
        Self {
            half: half.get(),
            step,
            pixels,
            first: 0,
            frames: VecDeque::new(),
            spare: Vec::new(),
        }
    }

    pub(crate) fn half(&self) -> usize {
        self.half
    }

    pub(crate) fn slot(&mut self) -> Prepared {
        self.spare.pop().unwrap_or_else(|| Prepared {
            units: vec![0; self.pixels],
            sum: 0,
        })
    }

    pub(crate) fn push(&mut self, frame: Prepared) {
        self.frames.push_back(frame);
    }

    pub(crate) fn pushed(&self) -> usize {
        self.first + self.frames.len()
    }

    pub(crate) fn release_before(&mut self, index: usize) {
        while self.first < index {
            let Some(frame) = self.frames.pop_front() else {
                return;
            };
            self.spare.push(frame);
            self.first += 1;
        }
    }

    pub(crate) fn frame(&self, index: usize) -> &Prepared {
        &self.frames[index - self.first]
    }

    pub(crate) fn sampled(&self, target: usize, last: usize) -> impl Iterator<Item = &Prepared> {
        let from = target.saturating_sub(self.half);
        let to = (target + self.half).min(last);
        (from..=to).step_by(self.step.get()).map(|index| self.frame(index))
    }
}
