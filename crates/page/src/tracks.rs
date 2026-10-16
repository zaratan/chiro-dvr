use crate::escape::{href, text};
use crate::labels::{Label, Labels};
use crate::model::{Track, Video};

#[cfg(test)]
mod tests;

pub(crate) fn render(video: &Video, labels: &dyn Labels) -> String {
    let say = |label: Label<'_>| text(&labels.label(&label));
    let rows: Vec<String> = video.tracks.iter().map(|track| row(video, track, labels)).collect();
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
            <th scope="col">{track}</th>
            <th scope="col" class="num">{start}</th>
            <th scope="col" class="num">{length}</th>
            <th scope="col">{verdict}</th>
            <th scope="col">{watch}</th>
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
        verdict = say(Label::ColumnVerdict),
        watch = say(Label::ColumnWatch),
        rows = rows.join("\n"),
    )
}

fn row(video: &Video, track: &Track, labels: &dyn Labels) -> String {
    let say = |label: Label<'_>| text(&labels.label(&label));
    let start = labels.label(&Label::Start(track.start_s));
    let total = video.tracks.len();
    let part = track
        .summary
        .as_ref()
        .and_then(|file| video.summaries.iter().position(|summary| &summary.file == file))
        .map(|index| format!(r#" data-part="{}""#, index + 1))
        .unwrap_or_default();
    let play = |file: &Option<String>, zoom: bool| {
        file.as_ref().map(|file| {
            let (label, shown) = if zoom {
                (Label::PlayZoom(track.id), Label::Zoom)
            } else {
                (Label::PlayClip(track.id), Label::Clip)
            };
            format!(
                r#"<button type="button" class="play" data-src="{src}" data-now="{now}" aria-label="{label}">{shown}</button>"#,
                src = href(&video.folder, file),
                now = say(Label::Playing {
                    track: track.id,
                    total,
                    start: &start,
                    zoom
                }),
                label = say(label),
                shown = say(shown),
            )
        })
    };
    let buttons: String = [play(&track.clip, false), play(&track.zoom, true)]
        .into_iter()
        .flatten()
        .collect();
    let dot = track
        .color
        .map(|[red, green, blue]| format!(r#" style="--dot: rgb({red} {green} {blue})""#))
        .unwrap_or_default();
    let verdict = track
        .verdict
        .as_deref()
        .filter(|verdict| !verdict.trim().is_empty())
        .map_or_else(
            || {
                format!(
                    r#"<td class="verdict empty">—<span class="sr"> {}</span></td>"#,
                    say(Label::NoVerdict)
                )
            },
            |verdict| format!(r#"<td class="verdict">{}</td>"#, text(verdict)),
        );
    format!(
        r#"            <tr{part}>
              <th scope="row"><button type="button" class="dot"{dot} aria-label="{show}">{id}</button></th>
              <td class="num">{start}</td>
              <td class="num">{duration}</td>
              {verdict}
              <td class="watch">{buttons}</td>
            </tr>"#,
        show = say(Label::ShowTrack(track.id)),
        id = track.id,
        start = text(&start),
        duration = say(Label::Seconds(track.duration_s)),
    )
}

fn file_name(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}
