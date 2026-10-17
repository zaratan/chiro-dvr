#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Evening {
    frames: Vec<usize>,
    spent_s: f64,
    computed_frames: usize,
}

impl Evening {
    pub const fn new(frames: Vec<usize>) -> Self {
        Self {
            frames,
            spent_s: 0.0,
            computed_frames: 0,
        }
    }

    pub fn computed(&mut self, position: usize, spent_s: f64) {
        if let Some(frames) = self.frames_at(position)
            && spent_s.is_finite()
            && spent_s >= 0.0
        {
            self.spent_s += spent_s;
            self.computed_frames += frames;
        }
    }

    pub fn left_after(&self, position: usize) -> Option<f64> {
        let frames = self.frames_after(position);
        if frames == 0 {
            return None;
        }
        Some(self.seconds_per_frame()? * frames as f64)
    }

    pub fn left_during(&self, position: usize, elapsed_s: f64) -> Option<f64> {
        let current = self.frames_at(position)?;
        let after = self.frames_after(position);
        if current + after == 0 {
            return None;
        }
        let per_frame = self.seconds_per_frame()?;
        Some((per_frame * current as f64 - elapsed_s).max(0.0) + per_frame * after as f64)
    }

    fn seconds_per_frame(&self) -> Option<f64> {
        (self.computed_frames > 0).then(|| self.spent_s / self.computed_frames as f64)
    }

    fn frames_at(&self, position: usize) -> Option<usize> {
        self.frames.get(position.checked_sub(1)?).copied()
    }

    fn frames_after(&self, position: usize) -> usize {
        self.frames.iter().skip(position).sum()
    }
}
