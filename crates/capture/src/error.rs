use std::time::Duration;

/// Why a capture failed.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// `chrome-headless-shell` is not installed.
    #[error("chrome-headless-shell not found: set DIAPO_CHROME or put it on PATH")]
    MissingChrome,
    /// The job cannot be captured as asked.
    #[error("invalid job: {0}")]
    Job(String),
    /// Chrome refused a command, crashed or closed its pipe.
    #[error("chrome: {0}")]
    Chrome(String),
    /// Chrome did not answer in time.
    #[error("chrome did not answer {0} within {1:?}")]
    Timeout(String, Duration),
    /// The page did not become a ready deck, or threw.
    #[error("deck: {0}")]
    Deck(String),
    /// The caller's progress callback asked to stop.
    #[error("capture cancelled")]
    Cancelled,
    /// A file or a pipe failed.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
