use crate::labels::{Label, Labels};
use crate::model::{Background, Layout, Summary, Trace, Track, Video};

#[derive(Debug)]
pub(crate) struct Named;

impl Labels for Named {
    fn label(&self, label: &Label<'_>) -> String {
        match label {
            Label::Language => "fr".to_owned(),
            other => format!("{other:?}"),
        }
    }
}

pub(crate) fn track(id: usize, summary: &str) -> Track {
    Track {
        id,
        start_s: 4.8 + id as f64,
        duration_s: 0.6,
        color: Some([255, 214, 10]),
        summary: Some(summary.to_owned()),
        clip: Some(format!("extraits/piste_{id:02}_0m04s.mp4")),
        zoom: None,
        start_xy: None,
        trace: None,
    }
}

pub(crate) fn summary(file: &str) -> Summary {
    Summary {
        file: file.to_owned(),
        width: 1879,
        height: 1215,
        layout: None,
    }
}

pub(crate) fn video(name: &str) -> Video {
    Video {
        name: name.to_owned(),
        folder: name.to_owned(),
        date: None,
        length_s: 40.4,
        not_analysed_s: 0.0,
        open: true,
        too_many: None,
        background: None,
        summaries: vec![summary("v_resume.png")],
        tracks: (1..=4).map(|id| track(id, "v_resume.png")).collect(),
        table: Some("v_pistes.csv".to_owned()),
        annotated: None,
    }
}

pub(crate) fn trace(id: usize) -> Trace {
    let y = 100 * id;
    Trace {
        points: format!("10,{y} 200,{y}"),
        head: Some(format!("230,{y} 200,{} 200,{}", y - 10, y + 10)),
        dot: [250, y as i64],
        anchor: None,
    }
}

pub(crate) fn layout() -> Layout {
    Layout {
        window_s: [0.0, 300.07],
        ignored_s: Vec::new(),
        damaged_s: Vec::new(),
        other_starts_s: Vec::new(),
        radius: 23.0,
    }
}

pub(crate) fn drawn_video(name: &str) -> Video {
    let mut video = video(name);
    video.background = Some(Background {
        file: "v_fond.png".to_owned(),
        width: 1440,
        height: 1080,
    });
    video.summaries[0].layout = Some(layout());
    for track in &mut video.tracks {
        track.trace = Some(trace(track.id));
    }
    video
}
