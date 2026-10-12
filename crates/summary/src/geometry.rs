use types::track::Track;

#[cfg(test)]
mod tests;

pub type Spot = [f64; 2];

#[derive(Debug, Clone, PartialEq)]
pub struct Trajectory {
    points: Vec<Spot>,
    cumulative: Vec<f64>,
}

impl Trajectory {
    pub fn new(points: Vec<Spot>) -> Self {
        let mut cumulative = Vec::with_capacity(points.len());
        let mut reached = 0.0;
        let mut previous = points.first().copied();
        for &point in &points {
            if let Some(from) = previous {
                reached += distance(from, point);
            }
            cumulative.push(reached);
            previous = Some(point);
        }
        Self { points, cumulative }
    }

    pub fn of(track: &Track) -> Self {
        Self::new(track.points().iter().map(|point| [point.x, point.y]).collect())
    }

    pub fn points(&self) -> &[Spot] {
        &self.points
    }

    pub fn total(&self) -> f64 {
        self.cumulative.last().copied().unwrap_or_default()
    }

    pub fn start(&self) -> Spot {
        self.points.first().copied().unwrap_or_default()
    }

    pub fn end(&self) -> Spot {
        self.points.last().copied().unwrap_or_default()
    }

    pub fn at(&self, length: f64) -> Spot {
        let after = self.cumulative.partition_point(|&reached| reached <= length);
        if after == 0 {
            return self.start();
        }
        if after >= self.points.len() {
            return self.end();
        }
        let (from, to) = (self.points[after - 1], self.points[after]);
        let (from_length, to_length) = (self.cumulative[after - 1], self.cumulative[after]);
        let share = (length - from_length) / (to_length - from_length);
        [from[0] + share * (to[0] - from[0]), from[1] + share * (to[1] - from[1])]
    }

    pub fn resampled(&self, step: f64) -> Vec<Spot> {
        let total = self.total();
        let steps = if step > 0.0 { (total / step).ceil() as usize } else { 0 };
        let mut spots: Vec<Spot> = (0..steps)
            .map(|index| index as f64 * step)
            .filter(|&length| length < total)
            .map(|length| self.at(length))
            .collect();
        spots.push(self.at(total));
        spots
    }

    pub fn nearest(&self, spot: Spot) -> Spot {
        let mut best = self.start();
        for pair in self.points.windows(2) {
            let candidate = projected(spot, pair[0], pair[1]);
            if distance(spot, candidate) < distance(spot, best) {
                best = candidate;
            }
        }
        best
    }

    pub fn end_direction(&self, span: f64) -> Option<[f64; 2]> {
        let back = self.at((self.total() - span).max(0.0));
        let end = self.end();
        let (dx, dy) = (end[0] - back[0], end[1] - back[1]);
        let norm = dx.hypot(dy);
        (norm > 0.0).then(|| [dx / norm, dy / norm])
    }
}

fn projected(spot: Spot, from: Spot, to: Spot) -> Spot {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let squared = dx * dx + dy * dy;
    if squared == 0.0 {
        return from;
    }
    let share = (((spot[0] - from[0]) * dx + (spot[1] - from[1]) * dy) / squared).clamp(0.0, 1.0);
    [from[0] + share * dx, from[1] + share * dy]
}

pub fn distance(from: Spot, to: Spot) -> f64 {
    (to[0] - from[0]).hypot(to[1] - from[1])
}

pub fn drawn(spot: Spot) -> draw::canvas::Point {
    draw::canvas::Point {
        x: spot[0] as f32,
        y: spot[1] as f32,
    }
}
