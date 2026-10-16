use crate::cards;
use crate::escape::text;
use crate::labels::{Label, Labels};
use crate::model::{Entry, Page};
use crate::section;

#[cfg(test)]
mod tests;

pub const GENERATOR: &str = r#"<meta name="generator" content="batdetect">"#;
const STYLE: &str = include_str!("assets/page.css");
const SCRIPT: &str = include_str!("assets/page.js");
const MARKER_WINDOW: usize = 1024;

pub fn is_ours(head: &[u8]) -> bool {
    let window = &head[..head.len().min(MARKER_WINDOW)];
    window.windows(GENERATOR.len()).any(|part| part == GENERATOR.as_bytes())
}

pub fn render(page: &Page, labels: &dyn Labels) -> String {
    let say = |label: Label<'_>| text(&labels.label(&label));
    let entries: Vec<String> = page
        .entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let anchor = format!("v{}", index + 1);
            match entry {
                Entry::Video(video) => section::render(video, &anchor, labels),
                Entry::Failed(failed) => cards::failed(failed, &anchor, labels),
                Entry::Unfinished { name } => {
                    cards::pending(name, &anchor, Label::UnfinishedBadge, Label::UnfinishedText, labels)
                }
                Entry::Waiting { name } => {
                    cards::pending(name, &anchor, Label::WaitingBadge, Label::WaitingText, labels)
                }
            }
        })
        .collect();
    let toggle = if page.entries.iter().any(has_section) {
        format!(
            r#"<p class="all"><button type="button" class="toggle-all" data-expand="{expand}" data-collapse="{collapse}">{expand}</button></p>"#,
            expand = say(Label::ExpandAll),
            collapse = say(Label::CollapseAll),
        )
    } else {
        String::new()
    };
    format!(
        r#"<!doctype html>
<html lang="{lang}">
<head>
<meta charset="utf-8">
{GENERATOR}
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light dark">
<title>{heading} — {title}</title>
<style>
{STYLE}</style>
</head>
<body>
<header class="top">
  <h1>{heading}</h1>
  <p class="folder">{folder}</p>
  <p class="tally">{tally}</p>
  <p class="reminder">{reminder}</p>
  {toggle}
</header>
<main>
{entries}
</main>
<script>
{SCRIPT}</script>
</body>
</html>
"#,
        lang = say(Label::Language),
        heading = say(Label::Heading),
        title = text(&page.title),
        folder = say(Label::Folder(&page.title)),
        tally = tally(page, labels),
        reminder = say(Label::Reminder),
        entries = entries.join("\n"),
    )
}

const fn has_section(entry: &Entry) -> bool {
    match entry {
        Entry::Video(_) => true,
        Entry::Failed(failed) => failed.previous.is_some(),
        Entry::Unfinished { .. } | Entry::Waiting { .. } => false,
    }
}

fn tally(page: &Page, labels: &dyn Labels) -> String {
    let count = |wanted: fn(&Entry) -> bool| page.entries.iter().filter(|entry| wanted(entry)).count();
    let done = count(|entry| matches!(entry, Entry::Video(video) if video.too_many.is_none()));
    let too_many = count(|entry| matches!(entry, Entry::Video(video) if video.too_many.is_some()));
    let failed = count(|entry| matches!(entry, Entry::Failed(_)));
    let unfinished = count(|entry| matches!(entry, Entry::Unfinished { .. }));
    let waiting = count(|entry| matches!(entry, Entry::Waiting { .. }));
    let first_failed = page
        .entries
        .iter()
        .position(|entry| matches!(entry, Entry::Failed(_)))
        .map(|index| index + 1);
    let mut parts = vec![span("count", &labels.label(&Label::Videos(page.entries.len())))];
    if done > 0 {
        parts.push(span("count", &labels.label(&Label::Done(done))));
    }
    if too_many > 0 {
        parts.push(span("count", &labels.label(&Label::TooManyCount(too_many))));
    }
    if let Some(first) = first_failed {
        parts.push(format!(
            r##"<a class="count bad" href="#v{first}">{}</a>"##,
            text(&labels.label(&Label::FailedCount(failed)))
        ));
    }
    if unfinished > 0 {
        parts.push(span("count", &labels.label(&Label::UnfinishedCount(unfinished))));
    }
    if waiting > 0 {
        parts.push(span("count", &labels.label(&Label::WaitingCount(waiting))));
    }
    parts.push(span("count strong", &labels.label(&Label::Tracks(page.tracks()))));
    parts.join("\n    ")
}

fn span(class: &str, content: &str) -> String {
    format!(r#"<span class="{class}">{}</span>"#, text(content))
}
