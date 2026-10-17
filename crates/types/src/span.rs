use crate::frame::FrameIndex;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Span {
    first: FrameIndex,
    last: FrameIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run<T> {
    pub span: Span,
    pub members: Vec<T>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("the span ends at frame {last} before it starts at frame {first}")]
    Reversed { first: FrameIndex, last: FrameIndex },
}

impl Span {
    pub fn new(first: FrameIndex, last: FrameIndex) -> Result<Self, Error> {
        if last < first {
            return Err(Error::Reversed { first, last });
        }
        Ok(Self { first, last })
    }

    pub fn single(frame: FrameIndex) -> Self {
        Self {
            first: frame,
            last: frame,
        }
    }

    #[must_use]
    pub fn hull(self, other: Self) -> Self {
        Self {
            first: self.first.min(other.first),
            last: self.last.max(other.last),
        }
    }

    #[must_use]
    pub fn widened(self, before: usize, after: usize) -> Self {
        Self {
            first: FrameIndex::new(self.first.get().saturating_sub(before)),
            last: FrameIndex::new(self.last.get().saturating_add(after)),
        }
    }

    pub fn intersection(self, other: Self) -> Option<Self> {
        let first = self.first.max(other.first);
        let last = self.last.min(other.last);
        (first <= last).then_some(Self { first, last })
    }

    pub fn first(self) -> FrameIndex {
        self.first
    }

    pub fn last(self) -> FrameIndex {
        self.last
    }

    pub fn contains(self, frame: FrameIndex) -> bool {
        self.first <= frame && frame <= self.last
    }

    pub fn frame_count(self) -> usize {
        (self.last.get() - self.first.get()).saturating_add(1)
    }
}

pub fn merged(spans: impl IntoIterator<Item = Span>) -> Vec<Span> {
    merged_by(spans, |span| *span).into_iter().map(|run| run.span).collect()
}

pub fn merged_by<T>(items: impl IntoIterator<Item = T>, span_of: impl Fn(&T) -> Span) -> Vec<Run<T>> {
    let mut sorted: Vec<T> = items.into_iter().collect();
    sorted.sort_by_key(|item| span_of(item).first);
    let mut runs: Vec<Run<T>> = Vec::new();
    for item in sorted {
        let span = span_of(&item);
        match runs.last_mut() {
            Some(run) if span.first.get() <= run.span.last.get().saturating_add(1) => {
                run.span = run.span.hull(span);
                run.members.push(item);
            }
            _ => runs.push(Run {
                span,
                members: vec![item],
            }),
        }
    }
    runs
}
