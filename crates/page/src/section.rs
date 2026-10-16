use crate::escape::{href, text};
use crate::labels::{Label, Labels};
use crate::model::Video;
use crate::tracks;

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
    format!(
        r#"<details class="video" id="{anchor}"{open}>
  <summary class="video-head">
    <span class="chevron" aria-hidden="true"></span>
    <h2 class="name">{name}</h2>{badge}
    <span class="meta">{meta}</span>
  </summary>{notice}
  <div class="body">
    <p class="sr announce" role="status"></p>
    <div class="stage">
      <div class="pictures">
        {pictures}
      </div>
      <div class="player" hidden data-unreadable="{unreadable}">
        <div class="bar">
          <button type="button" class="back"><span aria-hidden="true">← </span>{back}</button>
          <p class="now"></p>
          <span class="steps">
            <button type="button" class="step" data-step="-1" aria-label="{previous_clip}"><span aria-hidden="true">‹ </span>{previous}</button>
            <button type="button" class="step" data-step="1" aria-label="{next_clip}">{next}<span aria-hidden="true"> ›</span></button>
          </span>
        </div>
        <video controls preload="none" playsinline aria-label="{player}"></video>
      </div>
    </div>
    <div class="review">
      <div class="tracks">
{table}
      </div>
    </div>
  </div>
</details>"#,
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
        unreadable = say(Label::Unreadable),
        back = say(Label::Back),
        previous_clip = say(Label::PreviousClip),
        previous = say(Label::Previous),
        next_clip = say(Label::NextClip),
        next = say(Label::Next),
        player = say(Label::Player(&video.name)),
        table = tracks::render(video, labels),
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
            let alt = if parts > 1 {
                Label::PartAlt {
                    video: &video.name,
                    part,
                    parts,
                    first,
                    last,
                }
            } else {
                Label::SummaryAlt {
                    video: &video.name,
                    tracks: video.tracks.len(),
                }
            };
            let role = if parts > 1 {
                format!(r#" role="tabpanel" id="{anchor}-p{part}" aria-labelledby="{anchor}-t{part}""#)
            } else {
                String::new()
            };
            let link = href(&video.folder, &summary.file);
            format!(
                r#"<div class="panel"{role}{hidden}><a class="summary" href="{link}" target="_blank"><img src="{link}" width="{width}" height="{height}" alt="{alt}" loading="lazy"></a></div>"#,
                hidden = if index > 0 { " hidden" } else { "" },
                width = summary.width,
                height = summary.height,
                alt = text(&labels.label(&alt)),
            )
        })
        .collect();
    format!("{tabs}{}", panels.join("\n        "))
}

fn whole(seconds: f64) -> u64 {
    seconds.max(0.0).floor() as u64
}

fn rounded_up(seconds: f64) -> u64 {
    seconds.max(0.0).ceil() as u64
}
