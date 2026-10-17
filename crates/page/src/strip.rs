use crate::colour::colour;
use crate::escape::text;
use crate::labels::{Label, Labels, SceneText};
use crate::model::{Layout, Track};

#[cfg(test)]
mod tests;

const SECONDS_PER_MINUTE: f64 = 60.0;
const PERCENT: f64 = 100.0;
const LAST_LABEL_SHARE: f64 = 0.06;
const MINUTE_STEPS: [(u32, &str); 3] = [(2, "m2"), (5, "m5"), (10, "m10")];

#[derive(Debug, Clone, Copy)]
pub(crate) struct Strip<'a> {
    pub(crate) layout: &'a Layout,
    pub(crate) tracks: &'a [&'a Track],
}

pub(crate) fn render(strip: Strip<'_>, labels: &dyn Labels) -> String {
    let say = |scene: SceneText<'_>| text(&labels.label(&Label::Scene(scene)));
    let [start, end] = strip.layout.window_s;
    let length = end - start;
    if length.is_nan() || length <= 0.0 {
        return String::new();
    }
    let at = |seconds: f64| ((seconds - start) / length).clamp(0.0, 1.0) * PERCENT;
    let mut rows = Vec::new();
    let bands = [
        (&strip.layout.ignored_s, "ignored", SceneText::BandNotAnalysed),
        (&strip.layout.damaged_s, "damaged", SceneText::BandUnreadable),
    ];
    for (ranges, class, name) in bands {
        if ranges.is_empty() {
            continue;
        }
        let spans: Vec<String> = ranges
            .iter()
            .map(|&[from, to]| {
                let left = at(from);
                format!(r#"<span style="left:{left:.3}%;width:{:.3}%"></span>"#, at(to) - left)
            })
            .collect();
        rows.push(format!(
            r#"<span class="lane-name">{}</span><div class="lane lost {class}">{}</div>"#,
            say(name),
            spans.concat()
        ));
    }
    let others = strip
        .layout
        .other_starts_s
        .iter()
        .map(|&seconds| format!(r#"<span class="start other" style="left:{:.3}%"></span>"#, at(seconds)));
    let mine = strip.tracks.iter().filter_map(|track| {
        Some(format!(
            r#"<span class="start" data-id="{}" style="left:{:.3}%;--dot:{}" title="{}"></span>"#,
            track.id,
            at(track.start_s),
            colour(track.color?),
            say(SceneText::StartOf {
                track: track.id,
                start: track.start_s
            })
        ))
    });
    rows.push(format!(
        r#"<span class="lane-name"></span><div class="lane starts">{}</div>"#,
        others.chain(mine).collect::<Vec<_>>().concat()
    ));
    let first = (start / SECONDS_PER_MINUTE).ceil() as u32;
    let last = (end / SECONDS_PER_MINUTE).floor() as u32;
    let ticks = (first..=last)
        .map(|minute| {
            let seconds = f64::from(minute) * SECONDS_PER_MINUTE;
            let mut classes = vec!["tick"];
            classes.extend(
                MINUTE_STEPS
                    .iter()
                    .filter(|(step, _)| minute.is_multiple_of(*step))
                    .map(|(_, class)| *class),
            );
            if seconds <= start {
                classes.push("first");
            } else if (end - seconds) / length < LAST_LABEL_SHARE {
                classes.push("last");
            }
            format!(
                r#"<span class="{}" style="left:{:.3}%">{}</span>"#,
                classes.join(" "),
                (seconds - start) / SECONDS_PER_MINUTE * PERCENT,
                say(SceneText::Clock(seconds))
            )
        })
        .collect::<Vec<_>>()
        .concat();
    rows.push(format!(
        r#"<span class="lane-name"></span><div class="lane axis"><div class="unit" style="width:{:.3}%">{ticks}</div></div>"#,
        SECONDS_PER_MINUTE / length * PERCENT
    ));
    format!(r#"<div class="strip" aria-hidden="true">{}</div>"#, rows.concat())
}
