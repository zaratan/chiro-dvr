use std::ffi::OsString;
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use batdetect::jobs::video_extensions;
use clap_complete::{Shell, generate};

use crate::arguments::{self, PROGRAM, full_command};
use crate::doctor;
use crate::errors::report;
use crate::texts::{Fixed, Message, Texts};

#[cfg(test)]
mod tests;

pub(crate) const COMMAND: &str = "completion";
const ZSH: &str = "zsh";
const INPUTS_SPEC: &str = "':inputs -- ";
const INPUTS_SPEC_END: &str = ":_files' \\";
const INPUTS_FUNCTION: &str = "_batdetect_inputs";
const CALL: &str = "if [ \"$funcstack[1]\" = \"_batdetect\" ]; then";
const SAVE: &str = "mkdir -p ~/.zfunc && batdetect completion zsh > ~/.zfunc/_batdetect";
const FPATH: &str = "fpath=(~/.zfunc $fpath)";
const COMPINIT: &str = "autoload -Uz compinit && compinit";

pub(crate) fn run(arguments: &[OsString], texts: Texts) -> ExitCode {
    if arguments != [ZSH] {
        return report(
            texts,
            arguments::refusal(texts, &Message::Fixed(Fixed::CompletionZshOnly)),
        );
    }
    let mut out = io::stdout().lock();
    let written = out.write_all(zsh_script(texts).as_bytes()).and_then(|()| out.flush());
    let guided = if out.is_terminal() {
        writeln!(io::stderr().lock(), "{}", guide(texts))
    } else {
        Ok(())
    };
    if written.and(guided).is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

pub(crate) fn zsh_script(texts: Texts) -> String {
    let mut script = Vec::new();
    generate(Shell::Zsh, &mut full_command(texts), PROGRAM, &mut script);
    let mut lines = Vec::new();
    for line in String::from_utf8_lossy(&script).lines() {
        if line == CALL {
            lines.push(inputs_function(texts));
        }
        lines.push(inputs_spec(line));
    }
    lines.join("\n") + "\n"
}

fn inputs_spec(line: &str) -> String {
    match line
        .strip_prefix(INPUTS_SPEC)
        .and_then(|rest| rest.strip_suffix(INPUTS_SPEC_END))
    {
        Some(help) => format!("'*:inputs -- {help}:{INPUTS_FUNCTION}' \\"),
        None => line.to_owned(),
    }
}

fn inputs_function(texts: Texts) -> String {
    let command = |name: &str, fixed: Fixed| quoted(&format!("{name}:{}", texts.text(fixed)));
    format!(
        "(( $+functions[{INPUTS_FUNCTION}] )) ||\n\
         {INPUTS_FUNCTION}() {{\n\
         \x20   local ret=1\n\
         \x20   if (( CURRENT == 2 )); then\n\
         \x20       local -a commands; commands=({} {})\n\
         \x20       _describe -t commands {} commands && ret=0\n\
         \x20   fi\n\
         \x20   _files -g \"[^.]*.(#i)({})\" && ret=0\n\
         \x20   return ret\n\
         }}\n",
        command(doctor::COMMAND, Fixed::CompletionDoctor),
        command(COMMAND, Fixed::CompletionCompletion),
        quoted(&texts.text(Fixed::CompletionCommandsGroup)),
        video_extensions().join("|")
    )
}

fn quoted(text: &str) -> String {
    format!("'{}'", text.replace('\'', "'\\''"))
}

fn guide(texts: Texts) -> String {
    [
        texts.text(Fixed::CompletionWithHomebrew),
        format!("  {COMPINIT}"),
        texts.text(Fixed::CompletionWithoutHomebrew),
        format!("  {SAVE}"),
        texts.text(Fixed::CompletionThenProfile),
        format!("  {FPATH}"),
        format!("  {COMPINIT}"),
    ]
    .join("\n")
}
