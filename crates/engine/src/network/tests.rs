use std::num::NonZeroUsize;

use proptest::prelude::*;

use super::{Network, Networks, odd_even_merge_sort};

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
fn every_zero_one_input_of_up_to_sixteen_samples_gives_the_two_middle_values() {
    let networks = Networks::new();
    for inputs in 1..=16_usize {
        let network = networks.get(NonZeroUsize::new(inputs).unwrap());
        for bits in 0_u32..1 << inputs {
            let values: Vec<u16> = (0..inputs).map(|bit| u16::from(bits >> bit & 1 == 1)).collect();

            assert_eq!(middle(network, &values), sorted_middle(&values), "{inputs} {bits:b}");
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
    fn the_network_agrees_with_a_sort(values in prop::collection::vec(any::<u16>(), 1..=16)) {
        let network = Network::median(NonZeroUsize::new(values.len()).unwrap());

        prop_assert_eq!(middle(&network, &values), sorted_middle(&values));
    }
}
