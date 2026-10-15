use std::error::Error as _;
use std::process::ExitCode;

use batdetect::process::Wording;
use clap::Command;
use clap::builder::StyledStr;
use clap::builder::styling::Style;
use clap::error::{ContextKind, ContextValue, Error, ErrorFormatter, ErrorKind};
use types::language::Language;

use crate::style::{HELP, STYLES};
use crate::texts::{Fixed, Message, Texts};

#[cfg(test)]
mod tests;

const TAB: &str = "  ";
const ENGLISH_USAGE_HEADING: &str = "Usage:";

#[derive(Debug)]
pub(crate) struct French;

#[derive(Debug)]
pub(crate) struct English;

impl ErrorFormatter for French {
    fn format_error(error: &Error<Self>) -> StyledStr {
        rendered(Texts::new(Language::French), error)
    }
}

impl ErrorFormatter for English {
    fn format_error(error: &Error<Self>) -> StyledStr {
        rendered(Texts::new(Language::English), error)
    }
}

pub(crate) fn report(texts: Texts, error: Error) -> ExitCode {
    let code = u8::try_from(error.exit_code());
    let printed = match Wording::language(&texts) {
        Language::French => error.apply::<French>().print(),
        Language::English => error.apply::<English>().print(),
    };
    match (printed, code) {
        (Ok(()), Ok(code)) => ExitCode::from(code),
        _ => ExitCode::FAILURE,
    }
}

pub(crate) fn refusal(texts: Texts, message: &Message, command: &mut Command) -> Error {
    let mut error = Error::new(ErrorKind::ValueValidation).with_cmd(command);
    error.insert(ContextKind::Custom, ContextValue::String(texts.say(message)));
    error.insert(ContextKind::Usage, ContextValue::StyledStr(command.render_usage()));
    error
}

pub(crate) fn rendered<F: ErrorFormatter>(texts: Texts, error: &Error<F>) -> StyledStr {
    let error_style = STYLES.get_error();
    let mut styled = format!("{error_style}{}{error_style:#} ", texts.text(Fixed::ErrorLabel));
    styled.push_str(&sentence(texts, error));
    styled.push_str(&suggestions(texts, error));
    if let Some(ContextValue::StyledStr(usage)) = error.get(ContextKind::Usage) {
        styled.push_str("\n\n");
        styled.push_str(&usage_line(texts, usage));
    }
    let literal = STYLES.get_literal();
    let flag = format!("{literal}--{HELP}{literal:#}");
    let more = texts.say(&Message::MoreInformation { flag });
    StyledStr::from(format!("{styled}\n\n{more}\n"))
}

fn text<F: ErrorFormatter>(error: &Error<F>, kind: ContextKind) -> Option<&str> {
    match error.get(kind) {
        Some(ContextValue::String(text)) => Some(text),
        _ => None,
    }
}

fn styled(style: Style, text: &str) -> String {
    format!("{style}{text}{style:#}")
}

fn sentence<F: ErrorFormatter>(texts: Texts, error: &Error<F>) -> String {
    let (invalid, literal, valid) = (STYLES.get_invalid(), STYLES.get_literal(), STYLES.get_valid());
    if let Some(custom) = text(error, ContextKind::Custom) {
        return custom.to_owned();
    }
    let argument = text(error, ContextKind::InvalidArg);
    let value = text(error, ContextKind::InvalidValue);
    let message = match (error.kind(), argument, value) {
        (ErrorKind::UnknownArgument, Some(argument), _) => Message::UnexpectedArgument {
            argument: styled(*invalid, argument),
        },
        (ErrorKind::InvalidValue, Some(argument), Some("")) => Message::ValueRequired {
            argument: styled(*invalid, argument),
        },
        (ErrorKind::InvalidValue, Some(argument), Some(value)) => Message::InvalidValue {
            value: styled(*invalid, value),
            argument: styled(*literal, argument),
        },
        (ErrorKind::ValueValidation, Some(argument), Some(value)) => match error.source() {
            Some(cause) => Message::InvalidValueBecause {
                value: styled(*invalid, value),
                argument: styled(*literal, argument),
                cause: cause.to_string(),
            },
            None => Message::InvalidValue {
                value: styled(*invalid, value),
                argument: styled(*literal, argument),
            },
        },
        (ErrorKind::TooManyValues, Some(argument), Some(value)) => Message::UnexpectedValue {
            value: styled(*invalid, value),
            argument: styled(*literal, argument),
        },
        (ErrorKind::ArgumentConflict, Some(argument), _) if text(error, ContextKind::PriorArg) == Some(argument) => {
            Message::UsedTwice {
                argument: styled(*invalid, argument),
            }
        }
        (ErrorKind::MissingRequiredArgument, _, _) => match missing(texts, error) {
            Some(listed) => return listed,
            None => unknown(error),
        },
        (ErrorKind::InvalidUtf8, _, _) => Message::Fixed(Fixed::InvalidUtf8),
        _ => unknown(error),
    };
    let sentence = texts.say(&message);
    if error.kind() == ErrorKind::InvalidValue
        && let Some(ContextValue::Strings(values)) = error.get(ContextKind::ValidValue)
        && !values.is_empty()
    {
        let values: Vec<String> = values.iter().map(|value| styled(*valid, value)).collect();
        let listed = texts.say(&Message::PossibleValues {
            values: values.join(", "),
        });
        return format!("{sentence}\n{TAB}{listed}");
    }
    sentence
}

fn missing<F: ErrorFormatter>(texts: Texts, error: &Error<F>) -> Option<String> {
    let Some(ContextValue::Strings(missing)) = error.get(ContextKind::InvalidArg) else {
        return None;
    };
    let valid = STYLES.get_valid();
    let listed: Vec<String> = missing
        .iter()
        .map(|name| format!("\n{TAB}{}", styled(*valid, name)))
        .collect();
    Some(format!("{}{}", texts.text(Fixed::MissingArguments), listed.concat()))
}

fn unknown<F: ErrorFormatter>(error: &Error<F>) -> Message {
    let cause = error
        .kind()
        .as_str()
        .map_or_else(|| format!("{:?}", error.kind()), str::to_owned);
    Message::ArgumentsRefused { cause }
}

fn suggestions<F: ErrorFormatter>(texts: Texts, error: &Error<F>) -> String {
    let (invalid, valid) = (STYLES.get_invalid(), STYLES.get_valid());
    let tip = styled(*valid, &texts.text(Fixed::TipLabel));
    let mut tips = Vec::new();
    if let Some(suggestion) = text(error, ContextKind::SuggestedArg) {
        tips.push(Message::SimilarArgument {
            suggestion: styled(*valid, suggestion),
        });
    }
    if let Some(suggestion) = text(error, ContextKind::SuggestedValue) {
        tips.push(Message::SimilarValue {
            suggestion: styled(*valid, suggestion),
        });
    }
    if matches!(error.get(ContextKind::Suggested), Some(ContextValue::StyledStrs(_)))
        && let Some(argument) = text(error, ContextKind::InvalidArg)
    {
        tips.push(Message::ArgumentAsValue {
            argument: styled(*invalid, argument),
            escaped: styled(*valid, &format!("-- {argument}")),
        });
    }
    let written: Vec<String> = tips
        .iter()
        .map(|message| format!("\n{TAB}{tip} {}", texts.say(message)))
        .collect();
    if written.is_empty() {
        String::new()
    } else {
        format!("\n{}", written.concat())
    }
}

fn usage_line(texts: Texts, usage: &StyledStr) -> String {
    let style = STYLES.get_usage();
    let ansi = usage.ansi().to_string();
    let english = format!("{style}{ENGLISH_USAGE_HEADING}{style:#} ");
    let body = ansi.strip_prefix(&english).unwrap_or(&ansi);
    format!("{style}{}{style:#} {body}", texts.text(Fixed::UsageHeading))
}
