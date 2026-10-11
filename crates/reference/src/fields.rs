#[cfg(test)]
mod tests;

pub(crate) mod spans {
    use serde::de::Error;
    use serde::{Deserialize, Deserializer, Serializer};
    use types::frame::FrameIndex;
    use types::span::Span;

    pub(crate) fn serialize<S: Serializer>(spans: &[Span], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(spans.iter().map(|span| [span.first(), span.last()]))
    }

    pub(crate) fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<Span>, D::Error> {
        Vec::<[FrameIndex; 2]>::deserialize(deserializer)?
            .into_iter()
            .map(|[first, last]| Span::new(first, last).map_err(D::Error::custom))
            .collect()
    }
}
