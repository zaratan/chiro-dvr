#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Build {
    version: String,
    release: String,
    commit: String,
    rustc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Refusal {
    #[error(
        "this binary was built outside a git checkout or without rustc: its provenance is unknown, so it records nothing"
    )]
    Unknown,
    #[error("this binary was built from {described}, whose changes are not committed: it records nothing")]
    Dirty { described: String },
    #[error("this binary was built from {described}, after no release tag: it records nothing")]
    Untagged { described: String },
}

const DIRTY_SUFFIX: &str = "-dirty";
const TAG_PREFIX: char = 'v';

impl Build {
    pub fn of_this_binary() -> Result<Self, Refusal> {
        Self::clean(env!("RECORD_DESCRIBED"), env!("RECORD_COMMIT"), env!("RECORD_RUSTC"))
    }

    pub fn clean(described: &str, commit: &str, rustc: &str) -> Result<Self, Refusal> {
        if [described, commit, rustc].iter().any(|engraved| engraved.is_empty()) {
            return Err(Refusal::Unknown);
        }
        if described.ends_with(DIRTY_SUFFIX) {
            return Err(Refusal::Dirty {
                described: described.to_owned(),
            });
        }
        let version = described.strip_prefix(TAG_PREFIX).ok_or_else(|| Refusal::Untagged {
            described: described.to_owned(),
        })?;
        let tag = version.split_once('-').map_or(version, |(tag, _)| tag);
        Ok(Self {
            version: version.to_owned(),
            release: format!("{TAG_PREFIX}{tag}"),
            commit: commit.to_owned(),
            rustc: rustc.to_owned(),
        })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    /// The release tag the binary was built after, which names the reference it records.
    pub fn release(&self) -> &str {
        &self.release
    }

    pub fn commit(&self) -> &str {
        &self.commit
    }

    pub fn rustc(&self) -> &str {
        &self.rustc
    }
}
