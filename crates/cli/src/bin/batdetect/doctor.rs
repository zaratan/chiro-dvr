use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::Path;
use std::process::ExitCode;

use batdetect::checkup::{Checkup, ENOUGH_FREE_BYTES, Encoding, Space, checkup};
use media::encode::Codec;

use crate::arguments;
use crate::errors::report;
use crate::messages::{version, write_access};
use crate::texts::{Fixed, Message, Texts};

#[cfg(test)]
mod tests;

pub(crate) const COMMAND: &str = "doctor";
const BYTES_PER_GIGABYTE: f64 = 1e9;
const TENTHS: f64 = 10.0;
const CURRENT_FOLDER: &str = ".";

pub(crate) fn run(arguments: &[OsString], texts: Texts) -> ExitCode {
    if !arguments.is_empty() {
        return report(
            texts,
            arguments::refusal(texts, &Message::Fixed(Fixed::DoctorNoArgument)),
        );
    }
    let folder = env::current_dir().unwrap_or_else(|_| Path::new(CURRENT_FOLDER).to_path_buf());
    let found = checkup(&folder);
    let ffmpeg = media::library::version();
    let shown = writeln!(io::stdout().lock(), "{}", report_of(&found, version(), &ffmpeg, texts));
    match (shown, found.problems()) {
        (Ok(()), 0) => ExitCode::SUCCESS,
        _ => ExitCode::FAILURE,
    }
}

pub(crate) fn report_of(found: &Checkup, version: &str, ffmpeg: &str, texts: Texts) -> String {
    let folder = found.folder.display().to_string();
    let mut lines = vec![texts.say(&Message::DoctorVersion {
        version: version.to_owned(),
        ffmpeg: ffmpeg.to_owned(),
    })];
    lines.extend(
        encoding_lines(found.encoding)
            .into_iter()
            .map(|fixed| texts.text(fixed)),
    );
    match found.space {
        Space::Enough(bytes) => lines.push(texts.say(&Message::DoctorSpace {
            gigabytes: gigabytes(bytes, 0, texts),
            folder: folder.clone(),
        })),
        Space::Short(bytes) => {
            lines.push(texts.say(&Message::DoctorSpaceShort {
                gigabytes: gigabytes(bytes, 1, texts),
                folder: folder.clone(),
                least: gigabytes(ENOUGH_FREE_BYTES, 0, texts),
            }));
            lines.push(texts.text(Fixed::DoctorTodoSpace));
        }
        Space::Unknown => lines.push(texts.say(&Message::DoctorSpaceUnknown { folder: folder.clone() })),
    }
    match found.refused_writing {
        None => lines.push(texts.say(&Message::DoctorWritable { folder })),
        Some(access) => {
            lines.push(texts.say(&Message::DoctorUnwritable {
                folder,
                access: write_access(access),
            }));
            lines.push(texts.text(Fixed::DoctorTodoWriting));
        }
    }
    lines.push(match found.problems() {
        0 => texts.text(Fixed::DoctorReady),
        count => texts.say(&Message::DoctorProblems { count }),
    });
    lines.join("\n")
}

fn encoding_lines(encoding: Encoding) -> Vec<Fixed> {
    match encoding {
        Encoding::Preferred(Codec::VideoToolbox) => vec![Fixed::DoctorHardware],
        Encoding::Preferred(Codec::OpenH264) => vec![Fixed::DoctorSoftware],
        Encoding::Fallback => vec![Fixed::DoctorFallback, Fixed::DoctorTodoFallback],
        Encoding::Untried => vec![Fixed::DoctorUntried, Fixed::DoctorTodoUntried],
        Encoding::Broken => vec![Fixed::DoctorBroken, Fixed::DoctorTodoBroken],
    }
}

fn gigabytes(bytes: u64, tenths: u8, texts: Texts) -> String {
    let scale = TENTHS.powi(i32::from(tenths));
    let value = (bytes as f64 / BYTES_PER_GIGABYTE * scale).floor() / scale;
    texts.fixed(value, usize::from(tenths))
}
