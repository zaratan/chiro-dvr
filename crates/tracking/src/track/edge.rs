use types::detection::Detection;
use types::size::SourceSize;

#[cfg(test)]
mod tests;

const EDGE_MARGIN: f64 = 30.0;

pub(crate) fn at_edge(point: &Detection, size: SourceSize) -> bool {
    let width = f64::from(size.width().get());
    let height = f64::from(size.height().get());
    point.x.min(width - point.x).min(point.y).min(height - point.y) < EDGE_MARGIN
}
