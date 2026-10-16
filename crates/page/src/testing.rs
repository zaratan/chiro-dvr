use crate::labels::{Label, Labels};
use crate::model::{Summary, Track, Video};

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
        verdict: None,
    }
}

pub(crate) fn summary(file: &str) -> Summary {
    Summary {
        file: file.to_owned(),
        width: 1879,
        height: 1215,
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
        summaries: vec![summary("v_resume.png")],
        tracks: (1..=4).map(|id| track(id, "v_resume.png")).collect(),
        table: Some("v_pistes.csv".to_owned()),
        annotated: None,
    }
}
