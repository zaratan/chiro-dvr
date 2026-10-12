use types::span::Span;

#[cfg(test)]
mod tests;

pub(crate) fn coverage(spans: &[Span], frame_count: usize) -> Vec<bool> {
    let mut covered = vec![false; frame_count];
    for span in spans {
        let end = span.last().get().saturating_add(1).min(frame_count);
        if let Some(frames) = covered.get_mut(span.first().get()..end) {
            frames.fill(true);
        }
    }
    covered
}
