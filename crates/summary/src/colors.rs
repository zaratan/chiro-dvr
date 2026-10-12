use crate::geometry::{Spot, Trajectory, distance};

#[cfg(test)]
mod tests;

const NEIGHBOR_SAMPLES_PER_DISTANCE: f64 = 4.0;

pub fn path_distance(one: &[Spot], other: &[Spot]) -> f64 {
    one.iter()
        .flat_map(|&a| other.iter().map(move |&b| distance(a, b)))
        .fold(f64::INFINITY, f64::min)
}

pub fn assign_colors(paths: &[Trajectory], palette_size: usize, near: f64) -> Vec<usize> {
    let sampled: Vec<Vec<Spot>> = paths
        .iter()
        .map(|path| path.resampled(near / NEIGHBOR_SAMPLES_PER_DISTANCE))
        .collect();
    let mut colors: Vec<usize> = Vec::with_capacity(paths.len());
    let mut used = vec![0_usize; palette_size];
    for (index, path) in sampled.iter().enumerate() {
        let mut around = vec![0_usize; palette_size];
        for (earlier, &color) in sampled[..index].iter().zip(&colors) {
            if path_distance(path, earlier) < near
                && let Some(count) = around.get_mut(color)
            {
                *count += 1;
            }
        }
        let chosen = (0..palette_size)
            .min_by_key(|&color| (around[color], used[color], color))
            .unwrap_or_default();
        if let Some(count) = used.get_mut(chosen) {
            *count += 1;
        }
        colors.push(chosen);
    }
    colors
}
