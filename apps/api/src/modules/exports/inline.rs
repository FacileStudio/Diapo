//! An HTML export inlines the deck's assets. Each is encoded once, and the size of the file is
//! known before it is built.

use std::collections::HashMap;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use tronc::{Error, Result};

use super::build::Bundle;
use crate::modules::assets::name;

/// The largest file an export builds in memory. Past it the export fails with a reason instead
/// of taking the server's memory.
pub const MAX_BYTES: usize = 300 << 20;

pub(super) fn too_large(max: usize) -> Error {
    Error::too_large(format!(
        "the export would be larger than {} MiB: use fewer or smaller assets, or reference each one fewer times",
        max >> 20
    ))
}

/// Each asset the source names, encoded once as a `data:` URL, with how many times the source
/// names it. The count is an upper bound of the references, so it bounds the output.
fn inlined(source: &str, bundle: Bundle) -> HashMap<String, (String, usize)> {
    let named = |(name, (mime, bytes)): (String, (String, Vec<u8>))| {
        let count = source.matches(name.as_str()).count();
        (count > 0).then(|| {
            let url = format!("data:{mime};base64,{}", STANDARD.encode(&bytes));
            (name, (url, count))
        })
    };
    bundle.into_iter().filter_map(named).collect()
}

/// One offline HTML file: the compiled deck with every asset it has inlined as a `data:` URL.
/// Each asset is encoded once, and a file that would pass `max` bytes is refused before it is
/// built.
pub fn html(source: &str, project: &scene::Project, bundle: Bundle, max: usize) -> Result<Vec<u8>> {
    let urls = inlined(source, bundle);
    let inlined: usize = urls.values().map(|(url, count)| url.len().saturating_mul(*count)).sum();
    if inlined.saturating_add(source.len()) > max {
        return Err(too_large(max));
    }
    let inline = |path: &str| urls.get(name::referenced(path).0).map(|(url, _)| url.clone());
    let html = scene::compile(project, &scene::Options::new().assets(&inline)).into_bytes();
    match html.len() > max {
        true => Err(too_large(max)),
        false => Ok(html),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck(source: &str, size: usize) -> (scene::Project, Bundle) {
        let bundle = Bundle::from([("assets/dot.png".to_owned(), ("image/png".to_owned(), vec![b'x'; size]))]);
        (scene::parse(source).unwrap(), bundle)
    }

    #[test]
    fn an_html_export_inlines_the_assets_it_has_and_loads_nothing() {
        let source = "# One\n\n![](assets/dot.png)\n\n![](assets/missing.png)\n";
        let (project, bundle) = deck(source, 3);
        let html = String::from_utf8(html(source, &project, bundle, MAX_BYTES).unwrap()).unwrap();
        assert!(html.contains("src=\"data:image/png;base64,eHh4\""), "{html}");
        assert!(html.contains("src=\"assets/missing.png\"") && !html.contains("/api/"));
    }

    #[test]
    fn many_references_to_one_asset_are_refused_before_they_are_built() {
        let source = "![](assets/dot.png)\n\n".repeat(400);
        let (project, bundle) = deck(&source, 64 << 10);
        let refused = html(&source, &project, bundle, 4 << 20).unwrap_err();
        assert_eq!(refused.code, tronc::Code::ResourceExhausted, "{}", refused.message);
        let (project, bundle) = deck(&source, 1 << 10);
        let fits = html(&source, &project, bundle, 4 << 20).unwrap();
        assert!(fits.len() < 4 << 20 && String::from_utf8(fits).unwrap().matches("data:image/png").count() == 400);
    }
}
