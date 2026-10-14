pub(crate) const BLOCK: usize = 8;
const C1: f64 = 6.5025;
const C2: f64 = 58.5225;

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct Similarity {
    pub(crate) sum: f64,
    pub(crate) blocks: usize,
}

impl Similarity {
    pub(crate) fn mean(self) -> f64 {
        self.sum / self.blocks as f64
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Area {
    pub(crate) left: f64,
    pub(crate) top: f64,
    pub(crate) right: f64,
    pub(crate) bottom: f64,
}

impl Area {
    fn touches(&self, x: usize, y: usize) -> bool {
        let (left, top) = (x as f64, y as f64);
        let (right, bottom) = (left + BLOCK as f64, top + BLOCK as f64);
        left < self.right && self.left < right && top < self.bottom && self.top < bottom
    }
}

fn block_similarity(first: &[u8], second: &[u8], width: usize, x: usize, y: usize) -> f64 {
    let pixels = (BLOCK * BLOCK) as f64;
    let rows = (y..y + BLOCK).flat_map(|row| (x..x + BLOCK).map(move |column| row * width + column));
    let mut pairs = [(0.0, 0.0); BLOCK * BLOCK];
    for (pair, index) in pairs.iter_mut().zip(rows) {
        *pair = (f64::from(first[index]), f64::from(second[index]));
    }
    let mean_a = pairs.iter().map(|pair| pair.0).sum::<f64>() / pixels;
    let mean_b = pairs.iter().map(|pair| pair.1).sum::<f64>() / pixels;
    let (mut var_a, mut var_b, mut covariance) = (0.0, 0.0, 0.0);
    for (a, b) in &pairs {
        var_a += (a - mean_a) * (a - mean_a);
        var_b += (b - mean_b) * (b - mean_b);
        covariance += (a - mean_a) * (b - mean_b);
    }
    let (var_a, var_b, covariance) = (var_a / pixels, var_b / pixels, covariance / pixels);
    ((2.0 * mean_a * mean_b + C1) * (2.0 * covariance + C2))
        / ((mean_a * mean_a + mean_b * mean_b + C1) * (var_a + var_b + C2))
}

pub(crate) fn masked_similarity(
    first: &[u8],
    second: &[u8],
    width: usize,
    height: usize,
    masked: &[Area],
) -> Similarity {
    let mut similarity = Similarity::default();
    for y in (0..=height - BLOCK).step_by(BLOCK) {
        for x in (0..=width - BLOCK).step_by(BLOCK) {
            if masked.iter().any(|area| area.touches(x, y)) {
                continue;
            }
            similarity.sum += block_similarity(first, second, width, x, y);
            similarity.blocks += 1;
        }
    }
    similarity
}
