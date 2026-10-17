use crate::escape::{href, text};
use crate::judge;
use crate::labels::{Label, Labels, SceneText};
use crate::model::{Summary, Track, Video};
use crate::scene::{self, Scene};
use crate::strip::{self, Strip};
use crate::{caption, tracks};

#[cfg(test)]
mod tests;

const META_SEPARATOR: &str = " · ";

pub(crate) fn render(video: &Video, anchor: &str, labels: &dyn Labels) -> String {
    rendered(video, anchor, labels, video.open, None)
}

pub(crate) fn render_previous(video: &Video, anchor: &str, labels: &dyn Labels) -> String {
    rendered(
        video,
        anchor,
        labels,
        false,
        Some(labels.label(&Label::PreviousResults)),
    )
}

fn rendered(video: &Video, anchor: &str, labels: &dyn Labels, open: bool, previous: Option<String>) -> String {
    let say = |label: Label<'_>| text(&labels.label(&label));
    let mut meta: Vec<String> = previous.into_iter().collect();
    meta.extend(video.date.map(|date| labels.label(&Label::Computed(date))));
    meta.push(labels.label(&Label::Length(whole(video.length_s))));
    meta.push(labels.label(&Label::Tracks(video.tracks.len())));
    let lost = rounded_up(video.not_analysed_s);
    if lost > 0 {
        meta.push(labels.label(&Label::NotAnalysed(lost)));
    }
    let notice = video
        .too_many
        .map(|too_many| {
            format!(
                r#"
  <div class="notice"><p>{}</p><p>{}</p></div>"#,
                say(Label::TooMany {
                    video: &video.name,
                    count: too_many.count,
                    limit: too_many.limit
                }),
                say(Label::TooManyTodo)
            )
        })
        .unwrap_or_default();
    let viewable = video.viewable();
    format!(
        r#"<details class="{class}" id="{anchor}" data-video="{folder}"{open}>
  <summary class="video-head">
    <span class="chevron" aria-hidden="true"></span>
    <h2 class="name">{name}</h2>{badge}
    <span class="meta">{meta}<span class="judged"></span></span>
  </summary>{notice}
  <div class="body">
    <div class="stage">
      <div class="pictures">
        {pictures}
      </div>
{judge}
    </div>
    <div class="review">
      <a class="thumb" hidden target="_blank" aria-label="{complete}"></a>
      <div class="orphans" hidden></div>
      <div class="tracks">
{table}
      </div>
    </div>
  </div>
</details>"#,
        class = if viewable { "video" } else { "video no-viewer" },
        folder = text(&video.folder),
        open = if open { " open" } else { "" },
        badge = if video.too_many.is_some() {
            format!(
                r#"
    <span class="badge">{}</span>"#,
                say(Label::TooManyBadge)
            )
        } else {
            String::new()
        },
        name = text(&video.name),
        meta = text(&meta.join(META_SEPARATOR)),
        pictures = pictures(video, anchor, labels),
        judge = if viewable {
            judge::render(video, labels)
        } else {
            String::new()
        },
        table = tracks::render(video, viewable, labels),
        complete = say(Label::Scene(SceneText::CompleteNewTab)),
    )
}

fn pictures(video: &Video, anchor: &str, labels: &dyn Labels) -> String {
    let parts = video.summaries.len();
    let spans: Vec<(usize, usize)> = video
        .summaries
        .iter()
        .map(|summary| {
            let ids = video
                .tracks
                .iter()
                .filter(|track| track.summary.as_deref() == Some(summary.file.as_str()))
                .map(|track| track.id);
            (ids.clone().min().unwrap_or(0), ids.max().unwrap_or(0))
        })
        .collect();
    let tabs = if parts > 1 {
        let buttons: Vec<String> = spans
            .iter()
            .enumerate()
            .map(|(index, &(first, last))| {
                let part = index + 1;
                format!(
                    r#"<button type="button" role="tab" class="part" id="{anchor}-t{part}" aria-controls="{anchor}-p{part}" aria-selected="{selected}" tabindex="{tabindex}" aria-label="{label}"><span class="part-number">{part}</span><span class="part-range">{first}–{last}</span></button>"#,
                    selected = part == 1,
                    tabindex = if part == 1 { 0 } else { -1 },
                    label = text(&labels.label(&Label::PartTab { part, parts, first, last })),
                )
            })
            .collect();
        format!(
            r#"<div class="parts"><span class="parts-label" aria-hidden="true">{}</span><div role="tablist" aria-label="{}">{}</div></div>"#,
            text(&labels.label(&Label::Part)),
            text(&labels.label(&Label::Parts(&video.name))),
            buttons.concat()
        )
    } else {
        String::new()
    };
    let panels: Vec<String> = video
        .summaries
        .iter()
        .zip(&spans)
        .enumerate()
        .map(|(index, (summary, &(first, last)))| {
            let part = index + 1;
            let role = if parts > 1 {
                format!(r#" role="tabpanel" id="{anchor}-p{part}" aria-labelledby="{anchor}-t{part}""#)
            } else {
                String::new()
            };
            let hidden = if index > 0 { " hidden" } else { "" };
            let drawn = Panel {
                video,
                summary,
                part,
                parts,
                first,
                last,
            };
            let content = scene_panel(drawn, labels).unwrap_or_else(|| picture_panel(drawn, labels));
            format!(r#"<div class="panel"{role}{hidden}>{content}</div>"#)
        })
        .collect();
    format!("{tabs}{}", panels.join("\n        "))
}

#[derive(Debug, Clone, Copy)]
struct Panel<'a> {
    video: &'a Video,
    summary: &'a Summary,
    part: usize,
    parts: usize,
    first: usize,
    last: usize,
}

impl Panel<'_> {
    fn alt(&self, scene: bool) -> Label<'_> {
        let video = self.video.name.as_str();
        match (self.parts > 1, scene) {
            (true, false) => Label::PartAlt {
                video,
                part: self.part,
                parts: self.parts,
                first: self.first,
                last: self.last,
            },
            (true, true) => Label::Scene(SceneText::PartAlt {
                video,
                part: self.part,
                parts: self.parts,
                first: self.first,
                last: self.last,
            }),
            (false, false) => Label::SummaryAlt {
                video,
                tracks: self.video.tracks.len(),
            },
            (false, true) => Label::Scene(SceneText::Alt {
                video,
                tracks: self.video.tracks.len(),
            }),
        }
    }
}

fn picture_panel(drawn: Panel<'_>, labels: &dyn Labels) -> String {
    let Panel { video, summary, .. } = drawn;
    let link = href(&video.folder, &summary.file);
    format!(
        r#"<a class="summary" href="{link}" target="_blank"><img src="{link}" width="{width}" height="{height}" alt="{alt}" loading="lazy"></a>"#,
        width = summary.width,
        height = summary.height,
        alt = text(&labels.label(&drawn.alt(false))),
    )
}

fn scene_panel(drawn: Panel<'_>, labels: &dyn Labels) -> Option<String> {
    let Panel { video, summary, .. } = drawn;
    let background = video.background.as_ref()?;
    let layout = summary.layout.as_ref()?;
    let tracks: Vec<&Track> = video
        .tracks
        .iter()
        .filter(|track| track.summary.as_deref() == Some(summary.file.as_str()))
        .collect();
    let alt = labels.label(&drawn.alt(true));
    let scene = scene::render(Scene {
        folder: &video.folder,
        background,
        radius: layout.radius,
        tracks: &tracks,
        alt: &alt,
    });
    let strip = strip::render(
        Strip {
            layout,
            tracks: &tracks,
        },
        labels,
    );
    Some(format!(
        r#"<p class="scene-line"><span>{caption}</span> <a class="complete" href="{link}" target="_blank">{complete}<span class="sr"> {new_tab}</span></a></p>{scene}{strip}"#,
        caption = caption::render(layout, tracks.len(), labels),
        link = href(&video.folder, &summary.file),
        complete = text(&labels.label(&Label::Scene(SceneText::Complete))),
        new_tab = text(&labels.label(&Label::Scene(SceneText::NewTab))),
    ))
}

fn whole(seconds: f64) -> u64 {
    seconds.max(0.0).floor() as u64
}

fn rounded_up(seconds: f64) -> u64 {
    seconds.max(0.0).ceil() as u64
}
