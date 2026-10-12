use std::path::PathBuf;

use crate::case::Case;
use crate::comparison::Comparison;
use crate::error::Error;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Outcome {
    pub compared: Vec<Comparison>,
    pub refused: Vec<Refusal>,
    pub failed: Vec<(PathBuf, Error)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub recording: PathBuf,
    pub reason: String,
}

impl Outcome {
    pub fn of(cases: &[Case], answers: Vec<Result<Comparison, Error>>) -> Self {
        let mut outcome = Self {
            compared: Vec::new(),
            refused: Vec::new(),
            failed: Vec::new(),
        };
        for (case, answer) in cases.iter().zip(answers) {
            let recording = case.recording().to_path_buf();
            match answer {
                Ok(comparison) => outcome.compared.push(comparison),
                Err(error) => match error.unsupported() {
                    Some(reason) => outcome.refused.push(Refusal {
                        recording,
                        reason: reason.to_string(),
                    }),
                    None => outcome.failed.push((recording, error)),
                },
            }
        }
        outcome
    }

    pub fn holds(&self, refusals_allowed: bool) -> bool {
        !self.compared.is_empty()
            && self.failed.is_empty()
            && (refusals_allowed || self.refused.is_empty())
            && self.compared.iter().all(|comparison| comparison.detections.holds())
    }
}
