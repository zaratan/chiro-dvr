use crate::geometry::{Spot, Trajectory};

#[cfg(test)]
mod tests;

const ARROW_HALF_ANGLE_DEGREES: f64 = 25.0;
const END_SPAN_FRACTION: f64 = 0.03;

pub fn arrow_head(path: &Trajectory, length: f64) -> Option<[Spot; 3]> {
    let span = (END_SPAN_FRACTION * path.total()).max(length);
    let [dx, dy] = path.end_direction(span)?;
    let tip = path.end();
    let backwards = dy.atan2(dx) + std::f64::consts::PI;
    let half = ARROW_HALF_ANGLE_DEGREES.to_radians();
    let wing = |side: f64| {
        let angle = backwards + side;
        [tip[0] + length * angle.cos(), tip[1] + length * angle.sin()]
    };
    Some([tip, wing(-half), wing(half)])
}
