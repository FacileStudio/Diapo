//! Where the compiled document finds the project's files.

use crate::references::{local, references};

/// Maps a project file to the URL the compiled document loads it from.
///
/// The CLI answers a `data:` URI to build one offline file; the web app answers the route
/// that serves the asset. `None` keeps the path as written.
pub trait Assets {
    /// The URL for the file at `path`, a relative path as the source wrote it.
    fn url(&self, path: &str) -> Option<String>;
}

impl<F: Fn(&str) -> Option<String>> Assets for F {
    fn url(&self, path: &str) -> Option<String> {
        self(path)
    }
}

fn escape(url: &str) -> String {
    url.replace('&', "&amp;").replace('"', "&quot;").replace('<', "&lt;")
}

/// Rewrites every project file `html` loads through `assets`.
pub(crate) fn rewrite(html: &str, assets: &dyn Assets) -> String {
    let mut out = String::with_capacity(html.len());
    let mut done = 0;
    for reference in references(html).into_iter().filter(|reference| local(reference.url)) {
        let Some(url) = assets.url(reference.url) else { continue };
        out.push_str(&html[done..reference.span.start]);
        out.push_str(&escape(&url));
        done = reference.span.end;
    }
    out.push_str(&html[done..]);
    out
}
