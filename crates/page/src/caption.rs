use crate::escape::text;
use crate::labels::{Label, Labels, SceneText};
use crate::model::Layout;

#[cfg(test)]
mod tests;

const MAX_PERIODS: usize = 3;
const PART_SEPARATOR: &str = " · ";
const PERIOD_SEPARATOR: &str = ", ";

#[derive(Debug, Clone, Copy)]
enum Voice {
    Shown,
    Aloud,
}

impl Voice {
    fn time(self, seconds: f64, labels: &dyn Labels) -> String {
        match self {
            Self::Shown => labels.label(&Label::Scene(SceneText::Clock(seconds))),
            Self::Aloud => labels.label(&Label::Aloud(seconds)),
        }
    }

    fn span(self, [start, end]: [u64; 2], labels: &dyn Labels) -> String {
        let (start, end) = (self.time(start as f64, labels), self.time(end as f64, labels));
        let (start, end) = (start.as_str(), end.as_str());
        labels.label(&Label::Scene(match self {
            Self::Shown => SceneText::Span { start, end },
            Self::Aloud => SceneText::SpanAloud { start, end },
        }))
    }
}

pub(crate) fn render(layout: &Layout, tracks: usize, labels: &dyn Labels) -> String {
    format!(
        r#"<span aria-hidden="true">{}</span><span class="sr">{}</span>"#,
        text(&line(layout, tracks, labels, Voice::Shown)),
        text(&line(layout, tracks, labels, Voice::Aloud)),
    )
}

fn line(layout: &Layout, tracks: usize, labels: &dyn Labels, voice: Voice) -> String {
    let say = |scene: SceneText<'_>| labels.label(&Label::Scene(scene));
    let [start, end] = layout.window_s;
    let not_analysed = match periods(&layout.ignored_s, labels, voice) {
        None => say(SceneText::NothingNotAnalysed),
        Some((spans, hidden)) => more(say(SceneText::NotAnalysed(&spans)), hidden, labels),
    };
    let unreadable = match periods(&layout.damaged_s, labels, voice) {
        None => say(SceneText::NothingUnreadable),
        Some((spans, hidden)) => more(say(SceneText::Unreadable(&spans)), hidden, labels),
    };
    [
        say(SceneText::Window {
            start: &voice.time(start, labels),
            end: &voice.time(end, labels),
        }),
        labels.label(&Label::Tracks(tracks)),
        [not_analysed, unreadable].join(PERIOD_SEPARATOR),
    ]
    .join(PART_SEPARATOR)
}

fn periods(ranges: &[[f64; 2]], labels: &dyn Labels, voice: Voice) -> Option<(String, usize)> {
    let merged = whole_seconds(ranges);
    if merged.is_empty() {
        return None;
    }
    let spans: Vec<String> = merged
        .iter()
        .take(MAX_PERIODS)
        .map(|&range| voice.span(range, labels))
        .collect();
    Some((spans.join(PERIOD_SEPARATOR), merged.len().saturating_sub(MAX_PERIODS)))
}

fn more(said: String, hidden: usize, labels: &dyn Labels) -> String {
    if hidden == 0 {
        return said;
    }
    format!("{said} {}", labels.label(&Label::Scene(SceneText::More(hidden))))
}

fn whole_seconds(ranges: &[[f64; 2]]) -> Vec<[u64; 2]> {
    let mut rounded: Vec<[u64; 2]> = ranges
        .iter()
        .map(|[from, to]| [from.max(0.0).floor() as u64, to.max(0.0).ceil() as u64])
        .collect();
    rounded.sort_unstable();
    let mut merged: Vec<[u64; 2]> = Vec::with_capacity(rounded.len());
    for range in rounded {
        match merged.last_mut() {
            Some(last) if range[0] <= last[1] => last[1] = last[1].max(range[1]),
            _ => merged.push(range),
        }
    }
    merged
}
