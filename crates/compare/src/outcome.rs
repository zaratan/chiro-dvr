use std::path::PathBuf;

use crate::case::Case;
use crate::comparison::Comparison;
use crate::error::Error;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Outcome {
    pub compared: Vec<Comparison>,
    pub failed: Vec<(PathBuf, Error)>,
}

impl Outcome {
    pub fn of(cases: &[Case], answers: Vec<Result<Comparison, Error>>) -> Self {
        let mut outcome = Self {
            compared: Vec::new(),
            failed: Vec::new(),
        };
        for (case, answer) in cases.iter().zip(answers) {
            match answer {
                Ok(comparison) => outcome.compared.push(comparison),
                Err(error) => outcome.failed.push((case.recording().to_path_buf(), error)),
            }
        }
        outcome
    }

    pub fn holds(&self) -> bool {
        !self.compared.is_empty()
            && self.failed.is_empty()
            && self.compared.iter().all(|comparison| comparison.detections.holds())
    }
}
