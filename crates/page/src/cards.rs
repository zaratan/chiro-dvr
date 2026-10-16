use crate::escape::text;
use crate::labels::{Label, Labels};
use crate::model::Failed;
use crate::section;

#[cfg(test)]
mod tests;

pub(crate) fn failed(failed: &Failed, anchor: &str, labels: &dyn Labels) -> String {
    let say = |label: Label<'_>| text(&labels.label(&label));
    let date = failed
        .date
        .map(|date| format!(r#"<span class="meta">{}</span>"#, say(Label::Tried(date))))
        .unwrap_or_default();
    let detail = failed
        .detail
        .as_deref()
        .map(|detail| format!(r#"<p class="detail">{}</p>"#, say(Label::Detail(detail))))
        .unwrap_or_default();
    let previous_anchor = format!("{anchor}-previous");
    let (link, previous) = failed.previous.as_ref().map_or_else(Default::default, |video| {
        (
            format!(
                r##"<p><a class="previous" href="#{previous_anchor}">{}</a></p>"##,
                say(Label::PreviousResults)
            ),
            section::render_previous(video, &previous_anchor, labels),
        )
    });
    format!(
        r#"<section class="video failed" id="{anchor}" aria-labelledby="{anchor}-name">
  <div class="video-head">
    <span class="mark" aria-hidden="true">!</span>
    <h2 id="{anchor}-name" class="name">{name}</h2>
    <span class="badge">{badge}</span>
    {date}
  </div>
  <div class="failure">
    <p class="reason">{reason}</p>
    <p class="todo">{todo}</p>
    {detail}
    {link}
  </div>
</section>{previous}"#,
        name = text(&failed.name),
        badge = say(Label::FailedBadge),
        reason = say(Label::Failure {
            video: &failed.name,
            cause: &failed.cause,
            place: &failed.place
        }),
        todo = say(Label::Todo {
            cause: &failed.cause,
            folder: &failed.folder
        }),
    )
}

pub(crate) fn pending(name: &str, anchor: &str, badge: Label<'_>, explained: Label<'_>, labels: &dyn Labels) -> String {
    format!(
        r#"<section class="video pending" id="{anchor}" aria-labelledby="{anchor}-name">
  <div class="video-head">
    <h2 id="{anchor}-name" class="name">{name}</h2>
    <span class="badge">{badge}</span>
  </div>
  <div class="failure"><p>{explained}</p></div>
</section>"#,
        name = text(name),
        badge = text(&labels.label(&badge)),
        explained = text(&labels.label(&explained)),
    )
}
