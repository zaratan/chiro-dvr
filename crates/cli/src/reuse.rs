use std::fs;

use files::names::params_json;
use files::params::{Finish, Recorded, language_code, read_params};
use media::decode::ActiveThreads;
use serde_json::{Map, Value};
use types::language::Language;

use crate::jobs::Job;
use crate::options::Options;
use crate::process::Extent;
use crate::settings::Settings;

#[cfg(test)]
mod tests;

const MACHINE: &str = "threads";
const RENDER: &str = "RenderConfig";
const ENCODERS: &str = "encoders";
const UNCOMMITTED: &str = "-dirty";

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Earlier {
    pub finish: Finish,
    pub tracks: usize,
    pub limit: usize,
    pub extent: Extent,
}

#[derive(Debug, Clone, Copy)]
pub struct Run<'a> {
    pub options: &'a Options,
    pub version: &'a str,
    pub language: Language,
}

pub fn earlier(job: &Job, run: Run<'_>) -> Option<Earlier> {
    let recorded = read_params(&params_json(&job.dest))?;
    let video_bytes = fs::metadata(&job.video).ok()?.len();
    same_run(&recorded, run, video_bytes).then_some(Earlier {
        finish: recorded.state,
        tracks: recorded.track_count,
        limit: run.options.render.max_tracks(),
        extent: Extent {
            length_s: recorded.length_s,
            not_analysed_s: recorded.not_analysed_s,
        },
    })
}

fn same_run(recorded: &Recorded, run: Run<'_>, video_bytes: u64) -> bool {
    let settings = Settings::of(run.options, &[], ActiveThreads::Single, 0);
    let Ok(Value::Object(current)) = serde_json::to_value(settings) else {
        return false;
    };
    !run.version.ends_with(UNCOMMITTED)
        && recorded.version == run.version
        && recorded.video_bytes == video_bytes
        && recorded.language == language_code(run.language)
        && same_settings(&recorded.rest, &current)
}

fn same_settings(recorded: &Map<String, Value>, current: &Map<String, Value>) -> bool {
    current
        .iter()
        .filter(|(key, _)| key.as_str() != MACHINE)
        .all(|(key, value)| {
            recorded
                .get(key)
                .is_some_and(|earlier| comparable(key, earlier) == comparable(key, value))
        })
}

fn comparable(key: &str, value: &Value) -> Value {
    let mut value = value.clone();
    if key == RENDER
        && let Some(render) = value.as_object_mut()
    {
        render.remove(ENCODERS);
    }
    value
}
