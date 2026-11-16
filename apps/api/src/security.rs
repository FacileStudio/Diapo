use std::path::Path;

use tronc::http::{SecurityConfig, Sources};

/// The CSP sources Diapo's client needs on top of tronc's policy: the inline scripts
/// adapter-static writes, hashed from the build the binary serves, and the two things a page
/// frames: the app's own sandboxed deck frame and the Bulle sidebar at `bulle`.
pub fn sources(client_dir: &Path, bulle: &str) -> Sources {
    let mut scripts = Vec::new();
    if tronc::spa::available(client_dir) {
        match tronc::spa::inline_script_hashes(client_dir) {
            Ok(hashes) => scripts.extend(hashes),
            Err(error) => tracing::error!(%error, "failed to hash the client's inline scripts"),
        }
    }
    let frames = vec!["'self'".to_owned(), bulle.to_owned()];
    Sources::from([("script-src".to_owned(), scripts), ("frame-src".to_owned(), frames)])
}

/// tronc's security configuration with [`sources`].
pub fn config(base: SecurityConfig, client_dir: &Path, bulle: &str) -> SecurityConfig {
    SecurityConfig { csp_sources: sources(client_dir, bulle), ..base }
}

#[cfg(test)]
mod tests {
    #[test]
    fn nothing_frames_diapo_and_diapo_frames_itself_and_bulle() {
        let sources = super::sources(std::path::Path::new("/nonexistent"), "https://bulle.test");
        let policy = tronc::http::build_csp(&sources, "").unwrap();
        let directive = |name: &str| policy.split(';').map(str::trim).find(|d| d.starts_with(name)).unwrap().to_owned();
        assert!(directive("frame-ancestors ").ends_with("'none'"), "{policy}");
        assert_eq!(directive("frame-src "), "frame-src 'self' https://bulle.test");
    }
}
