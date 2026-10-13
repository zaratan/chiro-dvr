#[cfg(test)]
mod tests;

pub fn median(values: impl IntoIterator<Item = f64>) -> Option<f64> {
    let mut sorted: Vec<f64> = values.into_iter().collect();
    sorted.sort_by(f64::total_cmp);
    let middle = sorted.len() / 2;
    match sorted.len() {
        0 => None,
        length if length.is_multiple_of(2) => Some(f64::midpoint(sorted[middle - 1], sorted[middle])),
        _ => Some(sorted[middle]),
    }
}
