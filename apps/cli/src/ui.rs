//! The five output levels of the suite CLI standard, and when they are colored.

use std::env;
use std::io::{self, IsTerminal};
use std::sync::atomic::{AtomicBool, Ordering};

/// Set by `--json`: status lines move to stderr so stdout holds one JSON document.
pub static JSON: AtomicBool = AtomicBool::new(false);

/// Set by `--no-color`.
pub static PLAIN: AtomicBool = AtomicBool::new(false);

/// Whether `--json` is on.
pub fn json() -> bool {
    JSON.load(Ordering::Relaxed)
}

fn paint(tty: bool, code: &str, text: &str) -> String {
    let plain = PLAIN.load(Ordering::Relaxed) || json() || !tty || env::var_os("NO_COLOR").is_some();
    if plain { text.to_owned() } else { format!("\x1b[{code}m{text}\x1b[0m") }
}

fn status(code: &str, glyph: &str, message: &str) {
    if json() {
        eprintln!("{} {message}", paint(io::stderr().is_terminal(), code, glyph));
    } else {
        println!("{} {message}", paint(io::stdout().is_terminal(), code, glyph));
    }
}

/// Work about to happen.
pub fn step(message: &str) {
    status("36", "\u{25b8}", message);
}

/// Work that completed.
pub fn success(message: &str) {
    status("32", "\u{2713}", message);
}

/// A line under the one it explains, indented two spaces.
pub fn hint(message: &str) {
    status("2", " ", message);
}

/// Degraded but continuing, on stderr.
pub fn warn(message: &str) {
    eprintln!("{} {message}", paint(io::stderr().is_terminal(), "33", "!"));
}

/// Aborting, on stderr.
pub fn error(message: &str) {
    eprintln!("{} {message}", paint(io::stderr().is_terminal(), "31", "\u{2717}"));
}
