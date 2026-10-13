use std::num::NonZeroU64;

use proptest::prelude::*;

use super::{SplitMix64, Xoshiro256, rejected};

#[test]
fn split_mix_from_zero_gives_the_first_outputs_of_its_authors() {
    let mut mix = SplitMix64::new(0);

    let first: Vec<u64> = (0..3).map(|_| mix.next_u64()).collect();

    assert_eq!(
        first,
        [0xE220_A839_7B1D_CDAF, 0x6E78_9E6A_A1B9_65F4, 0x06C4_5D18_8009_454F]
    );
}

#[test]
fn xoshiro_from_one_two_three_four_gives_the_outputs_of_its_reference_implementation() {
    let mut random = Xoshiro256 { state: [1, 2, 3, 4] };

    let first: Vec<u64> = (0..10).map(|_| random.next_u64()).collect();

    assert_eq!(
        first,
        [
            11_520,
            0,
            1_509_978_240,
            1_215_971_899_390_074_240,
            1_216_172_134_540_287_360,
            607_988_272_756_665_600,
            16_172_922_978_634_559_625,
            8_476_171_486_693_032_832,
            10_595_114_339_597_558_777,
            2_904_607_092_377_533_576,
        ]
    );
}

#[test]
fn two_seeds_give_two_streams() {
    let [mut one, mut two] = [1, 2].map(Xoshiro256::seeded);

    let streams = [0; 8].map(|_| (one.next_u64(), two.next_u64()));

    assert!(streams.iter().all(|(first, second)| first != second));
}

#[test]
fn the_rejected_draws_leave_the_same_number_of_draws_to_each_value() {
    for bound in [1, 2, 3, 7, 1 << 20, (1 << 63) + 1, u64::MAX] {
        let bound = NonZeroU64::new(bound).unwrap();

        let accepted = (1_u128 << 64) - u128::from(rejected(bound));

        assert_eq!(accepted % u128::from(bound.get()), 0, "{bound}");
        assert!(rejected(bound) < bound.get(), "{bound}");
    }
}

proptest! {
    #[test]
    fn a_uniform_draw_stays_between_its_bounds(seed: u64, low in -1e3..1e3_f64, width in 0.0..1e3_f64) {
        let mut random = Xoshiro256::seeded(seed);
        for _ in 0..64 {
            let drawn = random.uniform(low, low + width);
            prop_assert!((low..=low + width).contains(&drawn));
        }
    }

    #[test]
    fn a_draw_below_a_bound_stays_below_it(seed: u64, bound in 1..u64::MAX) {
        let mut random = Xoshiro256::seeded(seed);
        let bound = NonZeroU64::new(bound).unwrap();
        for _ in 0..64 {
            prop_assert!(random.below(bound) < bound.get());
        }
    }

    #[test]
    fn a_sign_is_minus_or_plus_one(seed: u64) {
        let mut random = Xoshiro256::seeded(seed);
        let signs: Vec<f64> = (0..64).map(|_| random.sign()).collect();
        prop_assert!(signs.iter().all(|sign| [-1.0, 1.0].contains(sign)));
    }
}

#[test]
fn both_signs_come_out_of_a_few_draws() {
    let mut random = Xoshiro256::seeded(1);

    let signs: Vec<f64> = (0..64).map(|_| random.sign()).collect();

    assert!(signs.contains(&-1.0) && signs.contains(&1.0));
}
