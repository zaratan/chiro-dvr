use render::config::RenderConfig;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, Default)]
pub struct Options {
    pub render: RenderConfig,
    pub annotated: bool,
}
