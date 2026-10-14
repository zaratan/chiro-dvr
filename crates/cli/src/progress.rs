use std::io;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Detection,
    Tracking,
    Summary,
    Clips,
}

impl Step {
    pub const ALL: [Self; 4] = [Self::Detection, Self::Tracking, Self::Summary, Self::Clips];
}

pub trait Progress {
    fn step(&self, step: Step) -> io::Result<()>;
}

#[derive(Debug, Clone, Copy)]
pub struct Silent;

impl Progress for Silent {
    fn step(&self, _step: Step) -> io::Result<()> {
        Ok(())
    }
}
