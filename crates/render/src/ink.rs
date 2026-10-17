use draw::color::{BOX, INK, Rgb, TRAIL};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Ink {
    pub(crate) mark: Rgb,
    pub(crate) trail: Rgb,
    pub(crate) rim: Option<Rgb>,
}

impl Ink {
    pub(crate) fn of(color: Option<Rgb>) -> Self {
        match color {
            Some(color) => Self {
                mark: color,
                trail: color,
                rim: Some(INK),
            },
            None => Self {
                mark: BOX,
                trail: TRAIL,
                rim: None,
            },
        }
    }
}

pub(crate) fn ink_at(inks: &[Ink], index: usize) -> Ink {
    inks.get(index).copied().unwrap_or_else(|| Ink::of(None))
}

pub(crate) fn inks(colors: &[Option<Rgb>], track_count: usize) -> Vec<Ink> {
    (0..track_count)
        .map(|index| Ink::of(colors.get(index).copied().flatten()))
        .collect()
}
