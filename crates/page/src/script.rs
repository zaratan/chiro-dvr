#[cfg(test)]
mod tests;

macro_rules! scripts {
    ($($variant:ident => $key:literal [$($placeholder:literal),*]),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Script {
            $($variant),*
        }

        impl Script {
            pub const ALL: &[Self] = &[$(Self::$variant),*];

            pub const fn key(self) -> &'static str {
                match self {
                    $(Self::$variant => $key),*
                }
            }

            pub const fn message_id(self) -> &'static str {
                match self {
                    $(Self::$variant => concat!("page-js-", $key)),*
                }
            }

            pub const fn placeholders(self) -> &'static [&'static str] {
                match self {
                    $(Self::$variant => &[$($placeholder),*]),*
                }
            }
        }
    };
}

scripts! {
    Bar => "bar" [],
    Yes => "yes" [],
    No => "no" [],
    Doubt => "doubt" [],
    KeyYes => "key-yes" [],
    KeyNo => "key-no" [],
    KeyDoubt => "key-doubt" [],
    KeySpecies => "key-species" [],
    ValueYes => "value-yes" [],
    ValueNo => "value-no" [],
    ValueDoubt => "value-doubt" [],
    ValueYesOther => "value-yes-other" [],
    ValueNoOther => "value-no-other" [],
    ToSee => "to-see" [],
    Other => "other" ["value"],
    Counts => "counts" ["yes", "no", "doubt", "todo"],
    Judged => "judged" ["done", "total"],
    Pending => "pending" ["count"],
    Downloaded => "downloaded" ["time"],
    Current => "current" [],
    Download => "download" [],
    Reload => "reload" [],
    ReadFrom => "read-from" ["name", "date", "count"],
    ReadFile => "read-file" ["name", "date"],
    StepsHeading => "steps-heading" [],
    StepOne => "step-one" ["folder"],
    StepTwo => "step-two" ["folder"],
    Understood => "understood" [],
    ReplaceUnsaved => "replace-unsaved" [],
    NotVerdicts => "not-verdicts" ["name"],
    DraftSame => "draft-same" ["time"],
    DraftOther => "draft-other" ["time"],
    DraftKeep => "draft-keep" [],
    DraftDrop => "draft-drop" [],
    DraftDropConfirm => "draft-drop-confirm" [],
    DownloadNow => "download-now" [],
    Cleared => "cleared" ["track"],
    Orphans => "orphans" ["count"],
    OrphansWhy => "orphans-why" [],
    Orphan => "orphan" ["start", "track", "verdict"],
    Nearest => "nearest" ["track", "start"],
    Absent => "absent" ["count"],
    TrackOf => "track-of" ["track", "total", "start"],
    JudgeTrack => "judge-track" ["track"],
    Announce => "announce" ["track", "verdict"],
    AnnounceSpecies => "announce-species" ["track", "verdict", "species"],
    Species => "species" [],
    SpeciesThenEnter => "species-then-enter" [],
    SpeciesOf => "species-of" ["track"],
    VerdictOf => "verdict-of" ["track"],
    View => "view" [],
    Clip => "clip" [],
    Zoom => "zoom" [],
    NoZoom => "no-zoom" [],
    NoClip => "no-clip" [],
    Picture => "picture" [],
    Chain => "chain" [],
    PreviousTrack => "previous-track" [],
    NextTrack => "next-track" [],
    NextToSee => "next-to-see" [],
    Blocked => "blocked" [],
    Unreadable => "unreadable" [],
    Changed => "changed" [],
    ColumnVideo => "column-video" [],
    ColumnTrack => "column-track" [],
    ColumnStart => "column-start" [],
    ColumnStartSeconds => "column-start-seconds" [],
    ColumnVerdict => "column-verdict" [],
    ColumnSpecies => "column-species" [],
    ColumnStartX => "column-start-x" [],
    ColumnStartY => "column-start-y" [],
    Separator => "separator" [],
    DecimalMark => "decimal-mark" [],
}
