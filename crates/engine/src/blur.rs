use crate::convert::real;
use crate::geometry::Geometry;

#[cfg(test)]
mod tests;

pub(crate) const KERNEL_SUM: u32 = 4096;
const REACH_PER_SIGMA: f64 = 4.0;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Blur {
    weights: Vec<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("a {width}×{height} work frame is narrower than the {kernel} pixels of the blur")]
    TooSmall { width: usize, height: usize, kernel: u64 },
}

impl Blur {
    pub(crate) fn new(sigma: f64, geometry: Geometry) -> Result<Self, Error> {
        let reach = (REACH_PER_SIGMA * sigma).ceil();
        let (width, height) = (geometry.width(), geometry.height());
        let kernel = 2.0 * reach + 1.0;
        if kernel > real(width.min(height)) {
            return Err(Error::TooSmall {
                width,
                height,
                kernel: pixels(kernel),
            });
        }
        let weights = shared(&exact(sigma, whole(reach)));
        Ok(Self { weights })
    }

    pub(crate) fn weights(&self) -> &[u32] {
        &self.weights
    }

    pub(crate) fn reach(&self) -> usize {
        self.weights.len() / 2
    }
}

fn exact(sigma: f64, reach: usize) -> Vec<f64> {
    let spread = 2.0 * sigma * sigma;
    let gaussian: Vec<f64> = (0..=2 * reach)
        .map(|tap| {
            if tap == reach {
                return 1.0;
            }
            let offset = real(tap.abs_diff(reach));
            (-offset * offset / spread).exp()
        })
        .collect();
    let total: f64 = gaussian.iter().sum();
    gaussian
        .iter()
        .map(|weight| weight / total * f64::from(KERNEL_SUM))
        .collect()
}

fn shared(exact: &[f64]) -> Vec<u32> {
    let mut weights: Vec<u32> = exact.iter().map(|&share| share.floor() as u32).collect();
    let centre = weights.len() / 2;
    let mut missing = KERNEL_SUM.saturating_sub(weights.iter().sum());
    if missing % 2 == 1 {
        weights[centre] += 1;
        missing -= 1;
    }
    let mut pairs: Vec<usize> = (0..centre).collect();
    pairs.sort_by(|&a, &b| fraction(exact[b]).total_cmp(&fraction(exact[a])).then(b.cmp(&a)));
    for tap in pairs.into_iter().take(missing as usize / 2) {
        weights[tap] += 1;
        weights[2 * centre - tap] += 1;
    }
    weights
}

fn fraction(share: f64) -> f64 {
    share - share.floor()
}

fn pixels(length: f64) -> u64 {
    length as u64
}

fn whole(reach: f64) -> usize {
    reach as usize
}
