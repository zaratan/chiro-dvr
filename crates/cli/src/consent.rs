use std::io::{self, BufRead, Write};

#[cfg(test)]
mod tests;

const YES: [&str; 5] = ["", "o", "O", "y", "Y"];
const NO: [&str; 2] = ["n", "N"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    Process,
    Refuse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Asking {
    pub skip_refused: bool,
    pub terminal: bool,
}

impl Asking {
    pub const fn asks(self) -> bool {
        self.terminal && !self.skip_refused
    }

    pub const fn shows_the_refusals_first(self) -> bool {
        self.terminal || self.skip_refused
    }

    pub fn choose(self, question: &str, input: &mut impl BufRead, output: &mut impl Write) -> io::Result<Choice> {
        if self.skip_refused {
            return Ok(Choice::Process);
        }
        if !self.terminal {
            return Ok(Choice::Refuse);
        }
        loop {
            write!(output, "{question} ")?;
            output.flush()?;
            let mut answer = String::new();
            if input.read_line(&mut answer)? == 0 {
                writeln!(output)?;
                return Ok(Choice::Refuse);
            }
            if YES.contains(&answer.trim()) {
                return Ok(Choice::Process);
            }
            if NO.contains(&answer.trim()) {
                return Ok(Choice::Refuse);
            }
        }
    }
}
