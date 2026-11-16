//! Opening a URL in the person's browser.

use std::env;
use std::process::{Command, Stdio};

/// Whether opening a browser can work here. Over SSH on Linux there is no display.
pub fn has_display() -> bool {
    !cfg!(target_os = "linux") || env::var_os("DISPLAY").is_some() || env::var_os("WAYLAND_DISPLAY").is_some()
}

/// Opens `url` in the default browser. Answers whether the opener ran.
pub fn open(url: &str) -> bool {
    let opener = if cfg!(target_os = "macos") { "open" } else { "xdg-open" };
    has_display()
        && Command::new(opener)
            .arg(url)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
}
