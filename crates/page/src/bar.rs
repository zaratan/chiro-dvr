use crate::base64;
use crate::escape::text;
use crate::json;
use crate::labels::{Label, Labels};
use crate::model::{Page, VerdictsFile};
use crate::script::Script;

#[cfg(test)]
mod tests;

pub(crate) fn render(page: &Page, labels: &dyn Labels) -> String {
    let say = |script: Script| text(&labels.label(&Label::Script(script)));
    format!(
        r#"<div class="savebar" role="region" aria-label="{bar}" data-folder="{folder}">
  <span class="counts"></span>
  <span class="state"></span>
  <span class="actions"><button type="button" class="primary download">{download}</button><button type="button" class="reload">{reload}</button><input type="file" class="reload-file" accept=".csv,text/csv" hidden></span>
  <div class="draft" hidden role="status"><p></p><button type="button" class="draft-keep primary"></button> <button type="button" class="draft-drop"></button></div>
  <div class="steps" hidden><p class="steps-heading" tabindex="-1">{heading}</p><ol><li class="step-one"></li><li class="step-two"></li></ol><button type="button" class="understood">{understood}</button></div>
</div>
<div class="bar-info"><p class="read-from" hidden></p><p class="bar-notice" hidden></p></div>"#,
        bar = say(Script::Bar),
        folder = text(&page.title),
        download = say(Script::Download),
        reload = say(Script::Reload),
        heading = say(Script::StepsHeading),
        understood = say(Script::Understood),
    )
}

pub(crate) fn data(page: &Page, labels: &dyn Labels) -> String {
    let texts = json::object(
        Script::ALL
            .iter()
            .map(|&script| (script.key(), labels.label(&Label::Script(script)))),
    );
    let file = page
        .verdicts
        .as_ref()
        .map(|file| embedded(file, labels))
        .unwrap_or_default();
    format!(
        r#"{file}<datalist id="species-names"></datalist>
<script type="application/json" id="texts">{texts}</script>
"#
    )
}

fn embedded(file: &VerdictsFile, labels: &dyn Labels) -> String {
    let date = file
        .modified
        .map(|date| text(&labels.label(&Label::Day(date))))
        .unwrap_or_default();
    format!(
        r#"<template id="verdicts-file" data-name="{name}" data-date="{date}" data-candidates="{candidates}" data-bytes="{bytes}"></template>
"#,
        name = text(&file.name),
        candidates = file.candidates,
        bytes = base64::encoded(&file.bytes),
    )
}
