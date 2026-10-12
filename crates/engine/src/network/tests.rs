use std::array;
use std::num::NonZeroUsize;

use proptest::prelude::*;

use super::{Network, Networks, odd_even_merge_sort};
use crate::threshold::LANES;

fn middle(network: &Network, values: &[u16]) -> (u16, u16) {
    let mut lanes: Vec<[u16; 1]> = values.iter().map(|&value| [value]).collect();
    network.order(&mut lanes);
    (lanes[network.low()][0], lanes[network.high()][0])
}

fn sorted_middle(values: &[u16]) -> (u16, u16) {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    (sorted[(values.len() - 1) / 2], sorted[values.len() / 2])
}

#[test]
fn every_zero_one_input_of_up_to_twenty_samples_gives_the_two_middle_values() {
    let networks = Networks::new(NonZeroUsize::new(20).unwrap());
    for inputs in 1..=20_usize {
        let network = networks.get(NonZeroUsize::new(inputs).unwrap());
        for first in (0_u32..1 << inputs).step_by(LANES) {
            let patterns: [u32; LANES] = array::from_fn(|lane| (first + lane as u32) % (1 << inputs));
            let mut lanes: Vec<[u16; LANES]> = (0..inputs)
                .map(|bit| patterns.map(|bits| u16::from(bits >> bit & 1 == 1)))
                .collect();

            network.order(&mut lanes);

            for (lane, bits) in patterns.iter().enumerate() {
                let zeros = inputs - bits.count_ones() as usize;
                let sorted = |index: usize| u16::from(index >= zeros);
                assert_eq!(
                    (lanes[network.low()][lane], lanes[network.high()][lane]),
                    (sorted(network.low()), sorted(network.high())),
                    "{inputs} {bits:b}"
                );
            }
        }
    }
}

#[test]
fn the_eleven_samples_of_the_binoculars_keep_only_the_comparisons_that_reach_the_middle() {
    let network = Network::median(NonZeroUsize::new(11).unwrap());
    let sorting = odd_even_merge_sort(16)
        .into_iter()
        .filter(|&(_, upper)| upper < 11)
        .count();

    assert!(network.pairs.len() < sorting, "{} {sorting}", network.pairs.len());
    assert_eq!((network.low(), network.high()), (5, 5));
}

#[test]
fn an_even_count_keeps_both_middle_values() {
    let network = Network::median(NonZeroUsize::new(4).unwrap());

    assert_eq!(middle(&network, &[9, 1, 7, 3]), (3, 7));
}

#[test]
fn every_lane_is_ordered_on_its_own() {
    let network = Network::median(NonZeroUsize::new(3).unwrap());
    let mut lanes = [[5, 1], [4, 2], [3, 3]];

    network.order(&mut lanes);

    assert_eq!(lanes[1], [4, 2]);
}

proptest! {
    #[test]
    fn the_network_agrees_with_a_sort(values in prop::collection::vec(any::<u16>(), 1..=128)) {
        let network = Network::median(NonZeroUsize::new(values.len()).unwrap());

        prop_assert_eq!(middle(&network, &values), sorted_middle(&values));
    }
}
