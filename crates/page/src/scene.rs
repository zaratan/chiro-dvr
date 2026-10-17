use crate::colour::colour;
use crate::escape::{href, text};
use crate::model::{Background, Trace, Track};

#[cfg(test)]
mod tests;

const LABEL_TO_RADIUS: f64 = 1.1;

#[derive(Debug, Clone, Copy)]
pub(crate) struct Scene<'a> {
    pub(crate) folder: &'a str,
    pub(crate) background: &'a Background,
    pub(crate) radius: f64,
    pub(crate) tracks: &'a [&'a Track],
    pub(crate) alt: &'a str,
}

struct Shape<'a> {
    id: usize,
    trace: &'a Trace,
    colour: String,
}

pub(crate) fn render(scene: Scene<'_>) -> String {
    let drawn: Vec<Shape<'_>> = scene
        .tracks
        .iter()
        .filter_map(|track| {
            Some(Shape {
                id: track.id,
                trace: track.trace.as_ref()?,
                colour: colour(track.color?),
            })
        })
        .collect();
    let layer = |class: &str, shapes: Vec<String>| format!(r#"<g class="{class}">{}</g>"#, shapes.concat());
    let outlines = drawn
        .iter()
        .map(|one| {
            format!(
                r#"<polyline data-id="{}" points="{}"/>"#,
                one.id,
                text(&one.trace.points)
            )
        })
        .collect();
    let lines = drawn
        .iter()
        .map(|one| {
            format!(
                r#"<polyline data-id="{}" points="{}" stroke="{}"/>"#,
                one.id,
                text(&one.trace.points),
                one.colour
            )
        })
        .collect();
    let heads = drawn
        .iter()
        .filter_map(|one| {
            let head = one.trace.head.as_ref()?;
            Some(format!(
                r#"<polygon data-id="{}" points="{}" fill="{}"/>"#,
                one.id,
                text(head),
                one.colour
            ))
        })
        .collect();
    let leaders = drawn
        .iter()
        .filter_map(|one| {
            let [x1, y1] = one.trace.anchor?;
            let [x2, y2] = one.trace.dot;
            let ends = format!(r#"x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}""#);
            Some(format!(
                r#"<g data-id="{}"><line class="halo" {ends}/><line {ends} stroke="{}"/></g>"#,
                one.id, one.colour
            ))
        })
        .collect();
    let dots = drawn
        .iter()
        .map(|one| {
            let [x, y] = one.trace.dot;
            format!(
                r#"<g data-id="{id}"><circle cx="{x}" cy="{y}" r="{radius:.1}" fill="{colour}"/><text x="{x}" y="{y}">{id}</text></g>"#,
                id = one.id,
                radius = scene.radius,
                colour = one.colour
            )
        })
        .collect::<Vec<_>>()
        .concat();
    let Background { file, width, height } = scene.background;
    format!(
        r#"<svg class="scene" viewBox="0 0 {width} {height}" width="{width}" height="{height}" role="img" aria-label="{alt}"><image href="{source}" width="{width}" height="{height}"/>{outlines}{lines}{heads}{leaders}<g class="dots" font-size="{size:.1}">{dots}</g></svg>"#,
        alt = text(scene.alt),
        source = href(scene.folder, file),
        outlines = layer("outlines", outlines),
        lines = layer("lines", lines),
        heads = layer("heads", heads),
        leaders = layer("leaders", leaders),
        size = scene.radius * LABEL_TO_RADIUS,
    )
}
