use std::num::NonZeroUsize;
use std::{array, iter};

use crate::config::MAX_BACKGROUND_SAMPLES;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Network {
    pairs: Vec<(usize, usize)>,
    low: usize,
    high: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Networks(Vec<Network>);

impl Network {
    pub(crate) fn median(inputs: NonZeroUsize) -> Self {
        let inputs = inputs.get();
        let (low, high) = ((inputs - 1) / 2, inputs / 2);
        let mut needed = vec![false; inputs];
        needed[low] = true;
        needed[high] = true;
        let mut pairs: Vec<(usize, usize)> = odd_even_merge_sort(inputs.next_power_of_two())
            .into_iter()
            .filter(|&(_, upper)| upper < inputs)
            .rev()
            .filter(|&(lower, upper)| {
                let feeds = needed[lower] || needed[upper];
                needed[lower] |= feeds;
                needed[upper] |= feeds;
                feeds
            })
            .collect();
        pairs.reverse();
        Self { pairs, low, high }
    }

    pub(crate) fn low(&self) -> usize {
        self.low
    }

    pub(crate) fn high(&self) -> usize {
        self.high
    }

    pub(crate) fn order<const LANES: usize>(&self, lanes: &mut [[u16; LANES]]) {
        for &(lower, upper) in &self.pairs {
            let (first, second) = (lanes[lower], lanes[upper]);
            lanes[lower] = array::from_fn(|lane| first[lane].min(second[lane]));
            lanes[upper] = array::from_fn(|lane| first[lane].max(second[lane]));
        }
    }
}

impl Networks {
    pub(crate) fn new() -> Self {
        Self(
            iter::successors(Some(NonZeroUsize::MIN), |inputs| inputs.checked_add(1))
                .take(MAX_BACKGROUND_SAMPLES)
                .map(Network::median)
                .collect(),
        )
    }

    pub(crate) fn get(&self, inputs: NonZeroUsize) -> &Network {
        &self.0[inputs.get() - 1]
    }
}

fn odd_even_merge_sort(size: usize) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    for merged in iter::successors(Some(1), |merged| Some(merged * 2)).take_while(|&merged| merged < size) {
        for distance in iter::successors(Some(merged), |distance| (*distance > 1).then_some(distance / 2)) {
            for start in (distance % merged..size - distance).step_by(2 * distance) {
                for offset in 0..distance.min(size - start - distance) {
                    let (lower, upper) = (start + offset, start + offset + distance);
                    if lower / (2 * merged) == upper / (2 * merged) {
                        pairs.push((lower, upper));
                    }
                }
            }
        }
    }
    pairs
}
