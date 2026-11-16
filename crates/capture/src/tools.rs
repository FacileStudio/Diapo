//! Where `chrome-headless-shell` lives, and whether it keeps its sandbox.

use std::path::{Path, PathBuf};

use crate::Error;

/// The browser one capture runs.
#[derive(Debug, Clone)]
pub struct Tools {
    /// `chrome-headless-shell`.
    pub chrome: PathBuf,
    /// Whether Chrome keeps its own sandbox. Off only where the host cannot grant it one, such as
    /// a CI runner that refuses user namespaces; never in the render service.
    pub sandbox: bool,
}

fn on_path(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path).map(|dir| dir.join(name)).find(|file| file.is_file())
}

fn revision(dir: &Path) -> u64 {
    let name = dir.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    name.strip_prefix("chromium_headless_shell-").and_then(|digits| digits.parse().ok()).unwrap_or(0)
}

fn playwright() -> Option<PathBuf> {
    let cache = match std::env::var_os("PLAYWRIGHT_BROWSERS_PATH") {
        Some(dir) => PathBuf::from(dir),
        None => PathBuf::from(std::env::var_os("HOME")?).join(".cache/ms-playwright"),
    };
    let entries = std::fs::read_dir(cache).ok()?.flatten().map(|entry| entry.path());
    let mut builds: Vec<PathBuf> = entries.filter(|path| revision(path) > 0).collect();
    builds.sort_by_key(|path| std::cmp::Reverse(revision(path)));
    let shell = |build: &PathBuf| {
        let platforms = std::fs::read_dir(build).ok()?.flatten();
        platforms.map(|entry| entry.path().join("chrome-headless-shell")).find(|file| file.is_file())
    };
    builds.iter().find_map(shell)
}

/// `chrome-headless-shell`: `DIAPO_CHROME` alone when it is set, else `PATH`, then the newest
/// build in Playwright's cache.
pub fn chrome() -> Option<PathBuf> {
    match std::env::var_os("DIAPO_CHROME").filter(|value| !value.is_empty()) {
        Some(path) => Some(PathBuf::from(path)).filter(|file| file.is_file()),
        None => on_path("chrome-headless-shell").or_else(playwright),
    }
}

impl Tools {
    /// Finds Chrome. `DIAPO_CHROME_SANDBOX=off` turns its sandbox off.
    pub fn locate() -> Result<Tools, Error> {
        let chrome = chrome().ok_or(Error::MissingChrome)?;
        let sandbox = std::env::var("DIAPO_CHROME_SANDBOX").map_or(true, |value| value != "off");
        Ok(Tools { chrome, sandbox })
    }
}
