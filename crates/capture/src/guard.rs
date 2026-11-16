//! Which URLs the page may load: files of its own bundle, and nothing else.

use std::path::{Path, PathBuf};

use url::Url;

/// Allows `file://` URLs that resolve inside one directory, symbolic links followed, and
/// `data:`, `blob:` and `about:` URLs. Counts what it refused.
#[derive(Debug)]
pub(crate) struct Guard {
    root: PathBuf,
    /// How many requests were refused.
    pub(crate) refused: u32,
}

impl Guard {
    /// A guard over `dir`, which must exist.
    pub(crate) fn new(dir: &Path) -> std::io::Result<Guard> {
        Ok(Guard { root: dir.canonicalize()?, refused: 0 })
    }

    /// The bundle directory, canonical.
    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    fn inside(&self, url: &Url) -> bool {
        let file = url.to_file_path().ok().and_then(|path| path.canonicalize().ok());
        file.is_some_and(|path| path.starts_with(&self.root))
    }

    /// Whether the page may load `url`.
    pub(crate) fn allows(&self, url: &str) -> bool {
        match Url::parse(url) {
            Ok(url) if url.scheme() == "file" => self.inside(&url),
            Ok(url) => matches!(url.scheme(), "data" | "blob" | "about"),
            Err(_) => false,
        }
    }

    /// Decides one request, counting a refusal.
    pub(crate) fn decide(&mut self, url: &str) -> bool {
        let allowed = self.allows(url);
        self.refused += u32::from(!allowed);
        allowed
    }
}

#[cfg(test)]
mod tests {
    use super::Guard;

    #[test]
    fn only_files_inside_the_bundle_load() {
        let bundle = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(bundle.path().join("index.html"), "").unwrap();
        std::fs::write(outside.path().join("secret.txt"), "").unwrap();
        std::os::unix::fs::symlink(outside.path().join("secret.txt"), bundle.path().join("link.txt")).unwrap();
        let mut guard = Guard::new(bundle.path()).unwrap();
        let inside = |name: &str| format!("file://{}/{name}", guard.root().display());
        assert!(guard.allows(&inside("index.html")) && guard.allows(&format!("{}?v=1#top", inside("index.html"))));
        assert!(guard.allows("data:text/plain,hi") && guard.allows("about:blank"));
        for refused in [inside("missing.png"), inside("link.txt"), inside("../../etc/hostname")] {
            assert!(!guard.allows(&refused), "{refused}");
        }
        for refused in ["file:///etc/hostname", "https://example.com/a.png", "http://127.0.0.1:4000/", "ws://x/", "no"]
        {
            assert!(!guard.allows(refused), "{refused}");
        }
        assert!(guard.decide("data:,") && !guard.decide("https://example.com/") && guard.refused == 1);
    }
}
