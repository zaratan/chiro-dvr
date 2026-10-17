use std::env;
use std::ffi::OsString;
use std::process::ExitCode;

use crate::texts::Texts;
use crate::{completion, doctor};

pub(crate) fn other_command(texts: Texts) -> Option<ExitCode> {
    let mut arguments = env::args_os().skip(1);
    let first = arguments.next()?;
    let rest: Vec<OsString> = arguments.collect();
    match first.to_str()? {
        completion::COMMAND => Some(completion::run(&rest, texts)),
        doctor::COMMAND => Some(doctor::run(&rest, texts)),
        _ => None,
    }
}
