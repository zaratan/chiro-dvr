use types::detection::{Detection, Strength};
use types::frame::FrameIndex;

#[cfg(test)]
mod tests;

const BOX_SPREAD_LIMIT: f64 = 4.0;

pub(crate) fn bounded(points: &[Detection]) -> Vec<Detection> {
    let widest = median(points.iter().map(|point| point.width)).map(|width| BOX_SPREAD_LIMIT * width);
    let tallest = median(points.iter().map(|point| point.height)).map(|height| BOX_SPREAD_LIMIT * height);
    points
        .iter()
        .map(|point| {
            let mut point = *point;
            if let Some(widest) = widest.filter(|widest| point.width > *widest) {
                (point.left, point.width) = (point.x - widest / 2.0, widest);
            }
            if let Some(tallest) = tallest.filter(|tallest| point.height > *tallest) {
                (point.top, point.height) = (point.y - tallest / 2.0, tallest);
            }
            point
        })
        .collect()
}

fn median(values: impl Iterator<Item = f64>) -> Option<f64> {
    let mut values: Vec<f64> = values.collect();
    values.sort_by(f64::total_cmp);
    let upper = values.get(values.len() / 2).copied()?;
    if values.len() % 2 == 1 {
        return Some(upper);
    }
    let lower = values.get(values.len() / 2 - 1).copied()?;
    Some(f64::midpoint(lower, upper))
}

pub(crate) fn filled(points: &[Detection]) -> Vec<(Detection, bool)> {
    let mut filled = Vec::with_capacity(points.len());
    for (before, after) in points.iter().zip(points.iter().skip(1)) {
        filled.push((*before, false));
        let steps = after.frame.get().saturating_sub(before.frame.get());
        filled.extend((1..steps).map(|step| (between(before, after, step, steps), true)));
    }
    if let Some(last) = points.last() {
        filled.push((*last, false));
    }
    filled
}

fn between(before: &Detection, after: &Detection, step: usize, steps: usize) -> Detection {
    let share = step as f64 / steps as f64;
    let lerp = |from: f64, to: f64| from + (to - from) * share;
    Detection {
        frame: FrameIndex::new(before.frame.get() + step),
        x: lerp(before.x, after.x),
        y: lerp(before.y, after.y),
        left: lerp(before.left, after.left),
        top: lerp(before.top, after.top),
        width: lerp(before.width, after.width),
        height: lerp(before.height, after.height),
        area: lerp(before.area, after.area),
        amplitude: 0.0,
        strength: Strength::Strong,
    }
}
