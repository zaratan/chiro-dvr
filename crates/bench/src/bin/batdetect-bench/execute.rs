use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Instant;

use bench::against::{self, against};
use bench::collect::{self, Collected, Targets, collect};
use bench::folder;
use bench::metrics::{Evaluation, Judging, evaluate};
use bench::recorded;
use bench::report::{Seconds, bench_report, code_version};
use bench::run::Run;
use bench::stats::summarize;
use media::decode::Threads;
use reference::bench::{REPORT_FILE, read_targets, write_report};

use crate::arguments::{Request, Source};
use crate::messages;

pub(crate) const GUARD_FILE: &str = "guard.txt";

#[derive(Debug, thiserror::Error)]
pub(crate) enum Error {
    #[error("cannot create the folder {}", path.display())]
    Folder {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("the output folder {} holds the input of the bench, which would be overwritten", path.display())]
    Overwrite { path: PathBuf },
    #[error("cannot write {}", path.display())]
    Write {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error(transparent)]
    Run(#[from] folder::Error),
    #[error(transparent)]
    Collect(#[from] collect::Error),
    #[error(transparent)]
    Against(#[from] against::Error),
    #[error("the injected run cannot be tracked")]
    Tracking(#[from] tracking::error::Error),
    #[error(transparent)]
    File(#[from] reference::error::Error),
    #[error("the report cannot be encoded")]
    Encode(#[from] serde_json::Error),
    #[error("{} holds a target that cannot be replayed", path.display())]
    Target {
        path: PathBuf,
        #[source]
        source: recorded::Error,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Outcome {
    pub(crate) report: PathBuf,
    pub(crate) table: String,
    pub(crate) guard: Option<String>,
    pub(crate) failed: bool,
}

pub(crate) fn execute(request: &Request) -> Result<Outcome, Error> {
    let code = code_version();
    match &request.source {
        Source::Replay { folder } => replay(folder, &request.out_dir, request.settings, code),
        Source::Drawn { video, sampling } => {
            let targets = Targets::Drawn {
                sampling: sampling.clone(),
                track: request.settings.track,
                stability: request.settings.stability,
            };
            let collected = collect(video, Threads::Automatic, &request.detect, targets)?;
            measured(request, video, &collected, code)
        }
        Source::Targets { video, file } => {
            let parent = file.parent().filter(|parent| !parent.as_os_str().is_empty());
            refuse_overwrite(parent.unwrap_or(Path::new(".")), &destination(&request.out_dir, video))?;
            let targets = recorded::targets(&read_targets(file)?).map_err(|source| Error::Target {
                path: file.clone(),
                source,
            })?;
            let replayed = Targets::Replayed {
                path: file.clone(),
                targets,
            };
            let collected = collect(video, Threads::Automatic, &request.detect, replayed)?;
            measured(request, video, &collected, code)
        }
        Source::Against { video, folder } => {
            let dest = against_destination(&request.out_dir, video, folder);
            let checked = against(video, Threads::Automatic, &request.detect, request.settings, folder)?;
            let seconds = seconds(&checked.collected, checked.evaluation_time);
            let report = written(
                &checked.collected.run,
                &dest,
                request.settings,
                &checked.evaluation,
                seconds,
                code,
            )?;
            let guard = messages::guard(folder, &checked.guard);
            let guard_path = dest.join(GUARD_FILE);
            fs::write(&guard_path, format!("{guard}\n")).map_err(|source| Error::Write {
                path: guard_path,
                source,
            })?;
            Ok(Outcome {
                report,
                table: messages::table(&summarize(&checked.evaluation), &checked.evaluation),
                guard: Some(guard),
                failed: checked.guard.failed(),
            })
        }
    }
}

fn destination(out_dir: &Path, video: &Path) -> PathBuf {
    out_dir.join(video.file_stem().unwrap_or(video.as_os_str()))
}

fn against_destination(out_dir: &Path, video: &Path, folder: &Path) -> PathBuf {
    let mut name = video.file_stem().unwrap_or(video.as_os_str()).to_os_string();
    name.push("_");
    name.push(folder.file_name().unwrap_or(folder.as_os_str()));
    out_dir.join(name)
}

fn seconds(collected: &Collected, evaluation: std::time::Duration) -> Seconds {
    Seconds {
        reference: Some(collected.durations.reference),
        injected: Some(collected.durations.injected),
        evaluation,
    }
}

fn measured(request: &Request, video: &Path, collected: &Collected, code: String) -> Result<Outcome, Error> {
    let started = Instant::now();
    let evaluation = evaluate(&collected.run, request.settings)?;
    let seconds = seconds(collected, started.elapsed());
    let dest = destination(&request.out_dir, video);
    let report = written(&collected.run, &dest, request.settings, &evaluation, seconds, code)?;
    Ok(Outcome {
        report,
        table: messages::table(&summarize(&evaluation), &evaluation),
        guard: None,
        failed: false,
    })
}

fn written(
    run: &Run,
    dest: &Path,
    settings: Judging,
    evaluation: &Evaluation,
    seconds: Seconds,
    code: String,
) -> Result<PathBuf, Error> {
    folder::write(run, dest)?;
    let path = dest.join(REPORT_FILE);
    write_report(&path, &bench_report(run, settings, evaluation, code, seconds)?)?;
    Ok(path)
}

fn replay(folder: &Path, out_dir: &Path, settings: Judging, code: String) -> Result<Outcome, Error> {
    refuse_overwrite(folder, out_dir)?;
    let run = folder::read(folder)?;
    let started = Instant::now();
    let evaluation = evaluate(&run, settings)?;
    let seconds = Seconds {
        evaluation: started.elapsed(),
        ..Seconds::default()
    };
    fs::create_dir_all(out_dir).map_err(|source| Error::Folder {
        path: out_dir.to_path_buf(),
        source,
    })?;
    let path = out_dir.join(REPORT_FILE);
    write_report(&path, &bench_report(&run, settings, &evaluation, code, seconds)?)?;
    Ok(Outcome {
        report: path,
        table: messages::table(&summarize(&evaluation), &evaluation),
        guard: None,
        failed: false,
    })
}

fn refuse_overwrite(input: &Path, output: &Path) -> Result<(), Error> {
    match (fs::canonicalize(input), fs::canonicalize(output)) {
        (Ok(input), Ok(output)) if input == output => Err(Error::Overwrite { path: output }),
        _ => Ok(()),
    }
}
