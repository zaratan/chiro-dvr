use serde::{Deserialize, Serialize};
use types::span::Span;

use crate::fields;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Spans {
    #[serde(with = "fields::spans")]
    spans: Vec<Span>,
}

#[test]
fn spans_read_and_write_as_pairs_of_bounds() {
    let text = r#"{"spans":[[1286,1328],[6560,6617]]}"#;
    let read: Spans = serde_json::from_str(text).unwrap();
    assert_eq!(read.spans[1].frame_count(), 58);
    assert_eq!(serde_json::to_string(&read).unwrap(), text);
}

#[test]
fn a_reversed_span_is_refused() {
    let error = serde_json::from_str::<Spans>(r#"{"spans":[[10,9]]}"#).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("ends at frame 9 before it starts at frame 10")
    );
}
