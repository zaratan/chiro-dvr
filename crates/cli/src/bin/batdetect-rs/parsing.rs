use std::ffi::OsStr;
use std::marker::PhantomData;
use std::str::FromStr;

use clap::builder::TypedValueParser;
use clap::error::{ContextKind, ContextValue, Error, ErrorKind};
use clap::{Arg, Command};
use engine::settings::Bound;

#[cfg(test)]
mod tests;

const DECIMAL_COMMA: char = ',';
const DECIMAL_POINT: &str = ".";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Expected {
    Setting(Bound),
    Seconds,
    Region,
}

#[derive(Debug, thiserror::Error)]
#[error("{option} cannot read {written}")]
pub(crate) struct Unreadable {
    pub(crate) option: String,
    pub(crate) expected: Expected,
    pub(crate) written: String,
    pub(crate) pointed: Option<String>,
    #[source]
    pub(crate) cause: Box<dyn std::error::Error + Send + Sync>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Reader<T> {
    expected: Expected,
    decimal: bool,
    value: PhantomData<fn() -> T>,
}

impl<T> Reader<T> {
    pub(crate) const fn decimal(expected: Expected) -> Self {
        Self {
            expected,
            decimal: true,
            value: PhantomData,
        }
    }

    pub(crate) const fn exact(expected: Expected) -> Self {
        Self {
            expected,
            decimal: false,
            value: PhantomData,
        }
    }
}

impl<T> TypedValueParser for Reader<T>
where
    T: FromStr + Clone + Send + Sync + 'static,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    type Value = T;

    fn parse_ref(&self, command: &Command, arg: Option<&Arg>, value: &OsStr) -> Result<T, Error> {
        if value.is_empty() {
            return Err(empty_value(command, arg));
        }
        let (expected, decimal) = (self.expected, self.decimal);
        let option = arg
            .and_then(Arg::get_long)
            .map(|long| format!("--{long}"))
            .unwrap_or_default();
        let read = move |text: &str| {
            text.parse::<T>().map_err(|cause| Unreadable {
                option: option.clone(),
                expected,
                written: text.to_owned(),
                pointed: pointed(text, decimal),
                cause: Box::new(cause),
            })
        };
        read.parse_ref(command, arg, value)
    }
}

fn pointed(text: &str, decimal: bool) -> Option<String> {
    let pointed = text.replace(DECIMAL_COMMA, DECIMAL_POINT);
    (decimal && text.contains(DECIMAL_COMMA) && pointed.parse::<f64>().is_ok()).then_some(pointed)
}

fn empty_value(command: &Command, arg: Option<&Arg>) -> Error {
    let mut error = Error::new(ErrorKind::InvalidValue).with_cmd(command);
    let argument = arg.map(ToString::to_string).unwrap_or_default();
    error.insert(ContextKind::InvalidArg, ContextValue::String(argument));
    error.insert(ContextKind::InvalidValue, ContextValue::String(String::new()));
    error
}
