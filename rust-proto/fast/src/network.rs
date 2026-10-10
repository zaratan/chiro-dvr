//! Selection networks for the median: Batcher's odd-even merge sort, cut to `n` inputs
//! and pruned to the comparators that reach the middle outputs.

pub struct Network {
    pub pairs: Vec<(usize, usize)>,
    pub lo: usize,
    pub hi: usize,
}

fn batcher(size: usize) -> Vec<(usize, usize)> {
    let mut pairs = Vec::new();
    let mut p = 1;
    while p < size {
        let mut k = p;
        while k >= 1 {
            let mut j = k % p;
            while j + k < size {
                for i in 0..k.min(size - j - k) {
                    if (i + j) / (2 * p) == (i + j + k) / (2 * p) {
                        pairs.push((i + j, i + j + k));
                    }
                }
                j += 2 * k;
            }
            k /= 2;
        }
        p *= 2;
    }
    pairs
}

impl Network {
    pub fn median(n: usize) -> Self {
        assert!(n > 0);
        let (lo, hi) = ((n - 1) / 2, n / 2);
        let full: Vec<(usize, usize)> = batcher(n.next_power_of_two()).into_iter().filter(|&(_, j)| j < n).collect();
        let mut needed = vec![false; n];
        needed[lo] = true;
        needed[hi] = true;
        let mut kept = Vec::new();
        for &(i, j) in full.iter().rev() {
            if needed[i] || needed[j] {
                needed[i] = true;
                needed[j] = true;
                kept.push((i, j));
            }
        }
        kept.reverse();
        Self { pairs: kept, lo, hi }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(net: &Network, values: &mut [u16]) {
        for &(i, j) in &net.pairs {
            let (a, b) = (values[i], values[j]);
            values[i] = a.min(b);
            values[j] = a.max(b);
        }
    }

    #[test]
    fn every_zero_one_input_gives_the_sorted_middle_values() {
        for n in 1..=16 {
            let net = Network::median(n);
            for bits in 0u32..(1 << n) {
                let mut v: Vec<u16> = (0..n).map(|k| ((bits >> k) & 1) as u16).collect();
                let mut sorted = v.clone();
                sorted.sort_unstable();
                apply(&net, &mut v);
                assert_eq!((v[net.lo], v[net.hi]), (sorted[net.lo], sorted[net.hi]), "n={n} bits={bits:b}");
            }
        }
    }

    #[test]
    fn eleven_inputs_need_far_fewer_comparators_than_the_bubble_network() {
        assert!(Network::median(11).pairs.len() < 45);
    }
}
