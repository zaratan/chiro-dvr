use std::io::{self, IsTerminal};
use std::process::ExitCode;

use batdetect::consent::{Asking, Choice};
use batdetect::disk;
use batdetect::inventory::{Stock, take_stock};
use batdetect::jobs::{self, plan};
use batdetect::process::Wording as _;
use batdetect::reuse::{self, Run};
use batdetect::writable::unwritable_folders;

use crate::arguments::{self, Request};
use crate::errors::{self, report};
use crate::messages;
use crate::texts::{Fixed, Message, Texts};

pub(crate) fn admitted(request: &Request, texts: Texts) -> Result<Stock, ExitCode> {
    let (jobs, mut refused) = match plan(&request.inputs, request.out_dir.as_deref()) {
        Ok(jobs) => (jobs, Vec::new()),
        Err(jobs::Error::Refused { refused, valid }) => (valid, refused),
        Err(error) => {
            let (heading, lines) = messages::inputs_refused(texts, &error);
            return Err(report(texts, arguments::rejection(texts, &heading, &lines)));
        }
    };
    let run = Run {
        options: &request.options,
        version: messages::version(),
        language: texts.language(),
    };
    let mut stock = take_stock(jobs, |job| {
        if request.admission.force {
            None
        } else {
            reuse::earlier(job, run)
        }
    });
    let unwritable = unwritable_folders(&stock.jobs());
    if !unwritable.is_empty() {
        let (heading, lines) = messages::folders_refused(texts, &unwritable);
        return Err(report(texts, arguments::rejection(texts, &heading, &lines)));
    }
    refused.append(&mut stock.refused);
    stock.refused = refused;
    if !request.admission.ignore_disk {
        stock.refuse_short_disks(request.options.annotated, disk::volume);
    }
    if stock.refused.is_empty() {
        return Ok(stock);
    }
    let asking = Asking {
        skip_refused: request.admission.skip_refused,
        terminal: io::stdin().is_terminal(),
    };
    if stock.listed.is_empty() || !asking.shows_the_refusals_first() {
        let error = jobs::Error::Refused {
            valid: stock.jobs(),
            refused: stock.refused,
        };
        let (heading, lines) = messages::inputs_refused(texts, &error);
        return Err(report(texts, arguments::rejection(texts, &heading, &lines)));
    }
    let valid = stock.listed.len();
    for (heading, lines) in messages::inputs_skipped(texts, &stock.refused) {
        if errors::show(texts, arguments::rejection(texts, &heading, &lines)).is_err() {
            return Err(ExitCode::FAILURE);
        }
    }
    let question = texts.say(&Message::SkipRefusedQuestion { count: valid });
    match asking.choose(&question, &mut io::stdin().lock(), &mut io::stderr().lock()) {
        Ok(Choice::Process) => {
            stock.go_ahead_despite_the_disks();
            Ok(stock)
        }
        Ok(Choice::Refuse) | Err(_) => {
            let declined = Message::Fixed(Fixed::SkipRefusedDeclined);
            Err(report(texts, arguments::rejection(texts, &declined, &[])))
        }
    }
}
