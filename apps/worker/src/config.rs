//! What the worker reads from its environment, once, at start.

use std::path::PathBuf;
use std::time::Duration;

/// The caps the worker holds every job to, whatever the job file asks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Limits {
    /// `DIAPO_CAPTURE_MAX_SIDE`: the longest side of the stage in pixels, 4096 by default.
    pub side: u32,
    /// `DIAPO_CAPTURE_MAX_PAGES`: the most pictures or pages one job makes, 500 by default.
    pub pages: u32,
    /// `DIAPO_CAPTURE_TIMEOUT_SECONDS`: the wall clock one job gets, 300 by default.
    pub wall: Duration,
}

/// The worker's settings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Config {
    /// `DIAPO_CAPTURES_DIR`, `/data/captures` by default: one directory per job.
    pub captures: PathBuf,
    /// `DIAPO_CAPTURE_POLL_MS`: the pause between two looks at an empty queue, 500 by default.
    pub poll: Duration,
    pub limits: Limits,
}

fn number<T: std::str::FromStr>(key: &str, default: T) -> T {
    std::env::var(key).ok().and_then(|value| value.trim().parse().ok()).unwrap_or(default)
}

impl Default for Limits {
    fn default() -> Self {
        Limits { side: 4096, pages: 500, wall: Duration::from_secs(300) }
    }
}

impl Config {
    /// Reads the environment. A value that is not a number keeps the default.
    pub(crate) fn from_env() -> Config {
        let default = Limits::default();
        let limits = Limits {
            side: number("DIAPO_CAPTURE_MAX_SIDE", default.side),
            pages: number("DIAPO_CAPTURE_MAX_PAGES", default.pages),
            wall: Duration::from_secs(number("DIAPO_CAPTURE_TIMEOUT_SECONDS", default.wall.as_secs())),
        };
        let captures = std::env::var_os("DIAPO_CAPTURES_DIR").filter(|value| !value.is_empty());
        Config {
            captures: captures.map_or_else(|| PathBuf::from("/data/captures"), PathBuf::from),
            poll: Duration::from_millis(number("DIAPO_CAPTURE_POLL_MS", 500)),
            limits,
        }
    }
}
