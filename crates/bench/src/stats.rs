use crate::median::median;
use crate::metrics::{Evaluation, TargetResult};
use crate::synthetic::Class;

#[cfg(test)]
mod tests;

pub(crate) const WILSON_Z: f64 = 1.96;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClassSummary {
    pub class: Class,
    pub n: usize,
    pub found: usize,
    pub found_ci95: (f64, f64),
    pub median_completeness: f64,
    pub median_start_delay: Option<f64>,
    pub median_end_early: Option<f64>,
    pub mean_fragments: Option<f64>,
    pub median_effective: f64,
}

pub(crate) fn wilson(successes: usize, total: usize) -> (f64, f64) {
    if total == 0 {
        return (0.0, 0.0);
    }
    let n = total as f64;
    let p = successes as f64 / n;
    let z2 = WILSON_Z * WILSON_Z;
    let denominator = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denominator;
    let half = WILSON_Z * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt() / denominator;
    ((center - half).max(0.0), (center + half).min(1.0))
}

pub fn summarize(evaluation: &Evaluation) -> Vec<ClassSummary> {
    let mut groups: Vec<(Class, Vec<&TargetResult>)> = Vec::new();
    for result in &evaluation.targets {
        match groups.iter_mut().find(|(class, _)| *class == result.class) {
            Some((_, members)) => members.push(result),
            None => groups.push((result.class, vec![result])),
        }
    }
    groups.sort_by(|(left, _), (right, _)| left.order(*right));
    groups
        .into_iter()
        .map(|(class, members)| summary(class, &members))
        .collect()
}

fn summary(class: Class, members: &[&TargetResult]) -> ClassSummary {
    let found: Vec<&&TargetResult> = members.iter().filter(|result| result.found).collect();
    let as_real = |frames: usize| frames as f64;
    ClassSummary {
        class,
        n: members.len(),
        found: found.len(),
        found_ci95: wilson(found.len(), members.len()),
        median_completeness: median(members.iter().map(|result| result.completeness)).unwrap_or(0.0),
        median_start_delay: median(found.iter().filter_map(|result| result.start_delay).map(as_real)),
        median_end_early: median(found.iter().filter_map(|result| result.end_early).map(as_real)),
        mean_fragments: (!found.is_empty())
            .then(|| found.iter().map(|result| as_real(result.fragments)).sum::<f64>() / as_real(found.len())),
        median_effective: median(members.iter().map(|result| result.median_effective)).unwrap_or(0.0),
    }
}
