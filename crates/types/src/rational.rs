use std::fmt;
use std::num::{NonZeroI32, ParseIntError};
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Rational {
    num: i32,
    den: NonZeroI32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("a rational number cannot have a zero denominator")]
    ZeroDenominator,
    #[error("{num}/{den} does not fit in 32-bit terms once reduced")]
    OutOfRange { num: i32, den: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseError {
    #[error("{0:?} is not written as num/den")]
    NoSlash(String),
    #[error("a term of {text:?} is not a 32-bit integer")]
    Term {
        text: String,
        #[source]
        source: ParseIntError,
    },
    #[error(transparent)]
    Value(#[from] Error),
}

impl Rational {
    pub fn new(num: i32, den: i32) -> Result<Self, Error> {
        if den == 0 {
            return Err(Error::ZeroDenominator);
        }
        let divisor = i64::from(gcd(num.unsigned_abs(), den.unsigned_abs())) * i64::from(den.signum());
        let reduced_num = i32::try_from(i64::from(num) / divisor);
        let reduced_den = i32::try_from(i64::from(den) / divisor).ok().and_then(NonZeroI32::new);
        match (reduced_num, reduced_den) {
            (Ok(num), Some(den)) => Ok(Self { num, den }),
            _ => Err(Error::OutOfRange { num, den }),
        }
    }

    pub fn num(self) -> i32 {
        self.num
    }

    pub fn den(self) -> i32 {
        self.den.get()
    }

    pub fn to_f64(self) -> f64 {
        f64::from(self.num) / f64::from(self.den.get())
    }
}

impl fmt::Display for Rational {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}/{}", self.num, self.den)
    }
}

impl FromStr for Rational {
    type Err = ParseError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (num, den) = text
            .split_once('/')
            .ok_or_else(|| ParseError::NoSlash(text.to_owned()))?;
        let term = |term: &str| {
            term.parse().map_err(|source| ParseError::Term {
                text: text.to_owned(),
                source,
            })
        };
        Ok(Self::new(term(num)?, term(den)?)?)
    }
}

impl Serialize for Rational {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Rational {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer)?.parse().map_err(D::Error::custom)
    }
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}
