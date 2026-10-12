use types::track;

use crate::setting::Setting;

#[derive(Debug, Clone, Copy, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("{setting} is out of bounds: {value}")]
    Setting { setting: Setting, value: f64 },
    #[error("the association or the twin merge built an invalid track, which is a bug")]
    Track(#[from] track::Error),
}
