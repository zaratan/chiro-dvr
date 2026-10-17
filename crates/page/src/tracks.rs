use crate::colour::colour;
use crate::escape::{href, text};
use crate::labels::{Label, Labels, filled};
use crate::model::{Track, Video};
use crate::script::Script;

#[cfg(test)]
mod tests;

pub(crate) fn render(video: &Video, viewable: bool, labels: &dyn Labels) -> String {
    let say = |label: Label<'_>| text(&labels.label(&label));
    let rows: Vec<String> = video
        .tracks
        .iter()
        .map(|track| row(video, track, viewable, labels))
        .collect();
    let mut links = Vec::new();
    if let Some(table) = &video.table {
        links.push(format!(
            r#"<a href="{}">{}</a> <span class="file">{}</span>"#,
            href(&video.folder, table),
            say(Label::TableLink),
            text(file_name(table))
        ));
    }
    if let Some(annotated) = &video.annotated {
        links.push(format!(
            r#"<a href="{}">{}</a>"#,
            href(&video.folder, annotated),
            say(Label::AnnotatedLink)
        ));
    }
    let files = if links.is_empty() {
        String::new()
    } else {
        format!(r#"<p class="files">{}</p>"#, links.join("<br>"))
    };
    format!(
        r#"        <table>
          <caption class="sr">{caption}</caption>
          <thead><tr>
            <th scope="col" class="c-dot">{track}</th>
            <th scope="col" class="num c-start">{start}</th>
            <th scope="col" class="num c-length">{length}</th>{judging}
          </tr></thead>
          <tbody>
{rows}
          </tbody>
        </table>
        {files}"#,
        caption = say(Label::TableCaption(&video.name)),
        track = say(Label::ColumnTrack),
        start = say(Label::ColumnStart),
        length = say(Label::ColumnLength),
        judging = if viewable {
            format!(
                r#"
            <th scope="col">{}</th>
            <th scope="col" class="c-zoom"><span class="sr">{}</span></th>"#,
                say(Label::ColumnVerdict),
                say(Label::Script(Script::Zoom))
            )
        } else {
            String::new()
        },
        rows = rows.join("\n"),
    )
}

fn row(video: &Video, track: &Track, viewable: bool, labels: &dyn Labels) -> String {
    let say = |label: Label<'_>| text(&labels.label(&label));
    let part = track
        .summary
        .as_ref()
        .and_then(|file| video.summaries.iter().position(|summary| &summary.file == file))
        .map(|index| format!(r#" data-part="{}""#, index + 1))
        .unwrap_or_default();
    let point = track
        .start_xy
        .map(|[x, y]| format!(r#" data-x="{x}" data-y="{y}""#))
        .unwrap_or_default();
    let source = |attribute: &str, file: &Option<String>| {
        file.as_ref()
            .map(|file| format!(r#" {attribute}="{}""#, href(&video.folder, file)))
            .unwrap_or_default()
    };
    let colour = track.color.map(colour).unwrap_or_default();
    let dot = if colour.is_empty() {
        String::new()
    } else {
        format!(r#" style="--dot: {colour}""#)
    };
    let id = track.id.to_string();
    let judge = text(&filled(labels, Script::JudgeTrack, &[("track", &id)]));
    let (number, judging) = if viewable {
        let zoom = if track.zoom.is_some() {
            format!(r#"<span class="zoom-tag">{}</span>"#, say(Label::Script(Script::Zoom)))
        } else {
            String::new()
        };
        (
            format!(r#"<button type="button" class="dot"{dot} aria-label="{judge}">{id}</button>"#),
            format!(
                r#"
              <td class="verdict"><span class="chip todo">{}</span></td>
              <td class="zoom">{zoom}</td>"#,
                say(Label::Script(Script::ToSee))
            ),
        )
    } else {
        (format!(r#"<span class="dot"{dot}>{id}</span>"#), String::new())
    };
    format!(
        r#"            <tr data-id="{id}" data-start="{start_s:.2}"{point}{clip}{zoom_source}{part}>
              <th scope="row">{number}</th>
              <td class="num">{start}</td>
              <td class="num">{duration}</td>{judging}
            </tr>"#,
        start_s = track.start_s,
        clip = source("data-clip", &track.clip),
        zoom_source = source("data-zoom", &track.zoom),
        start = text(&labels.label(&Label::Start(track.start_s))),
        duration = say(Label::Seconds(track.duration_s)),
    )
}

fn file_name(path: &str) -> &str {
    path.rsplit_once('/').map_or(path, |(_, name)| name)
}
