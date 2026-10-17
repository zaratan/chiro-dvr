use types::detection::{Detection, FrameDetections, Measures};

#[cfg(test)]
mod tests;

pub(crate) const SUPPORT_FRAMES: usize = 3;
pub(crate) const SUPPORT_STEP: f64 = 60.0;
pub(crate) const SUPPORT_MOVE: f64 = 20.0;
pub(crate) const BRACKET_TOLERANCE: f64 = 10.0;
pub(crate) const BESIDE_STRONG: f64 = 12.0;

pub(crate) fn bracketed(detections: &FrameDetections) -> FrameDetections {
    let frames: Vec<&[Detection]> = detections.iter().map(|(_, found)| found).collect();
    let mut kept = FrameDetections::default();
    for (frame, found) in frames.iter().enumerate() {
        kept.push(
            found
                .iter()
                .filter(|detection| detection.strength.is_strong() || supported(&frames, frame, detection))
                .map(Measures::of),
        );
    }
    kept
}

fn supported(frames: &[&[Detection]], frame: usize, weak: &Detection) -> bool {
    let beside_strong = frames[frame]
        .iter()
        .any(|strong| strong.strength.is_strong() && strong.distance(weak) <= BESIDE_STRONG);
    if beside_strong {
        return false;
    }
    let after: Vec<(usize, &Detection)> = supports(frames, frame, weak, Direction::Ahead).collect();
    supports(frames, frame, weak, Direction::Behind).any(|(back, before)| {
        after
            .iter()
            .any(|&(ahead, next)| on_the_way(weak, (back, before), (ahead, next)))
    })
}

fn on_the_way(weak: &Detection, (back, before): (usize, &Detection), (ahead, next): (usize, &Detection)) -> bool {
    if before.distance(next) < SUPPORT_MOVE {
        return false;
    }
    let share = back as f64 / (back + ahead) as f64;
    let x = before.x + (next.x - before.x) * share;
    let y = before.y + (next.y - before.y) * share;
    (weak.x - x).hypot(weak.y - y) <= BRACKET_TOLERANCE
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Behind,
    Ahead,
}

fn supports<'a>(
    frames: &'a [&'a [Detection]],
    frame: usize,
    weak: &'a Detection,
    direction: Direction,
) -> impl Iterator<Item = (usize, &'a Detection)> + 'a {
    (1..=SUPPORT_FRAMES).flat_map(move |step| {
        let other = match direction {
            Direction::Behind => frame.checked_sub(step),
            Direction::Ahead => frame.checked_add(step),
        };
        other
            .and_then(|other| frames.get(other))
            .into_iter()
            .flat_map(|found| found.iter())
            .filter(move |strong| strong.strength.is_strong() && strong.distance(weak) <= SUPPORT_STEP * step as f64)
            .map(move |strong| (step, strong))
    })
}
