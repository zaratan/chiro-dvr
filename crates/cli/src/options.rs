use engine::config::DetectConfig;
use render::config::RenderConfig;
use tracking::stability::StabilityConfig;
use tracking::track::TrackConfig;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Options {
    pub detect: DetectConfig,
    pub track: TrackConfig,
    pub stability: StabilityConfig,
    pub render: RenderConfig,
    pub annotated: bool,
}
