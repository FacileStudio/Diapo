//! `diapo open`: the web page of a pulled deck.

use std::path::Path;

use serde_json::json;

use crate::cli::OpenArgs;
use crate::commands::print_json;
use crate::failure::{Failure, Outcome};
use crate::sync::Link;
use crate::url::encode;
use crate::{browser, ui};

/// The web page of the deck `link` names.
pub fn web_url(link: &Link) -> String {
    format!("{}/d/{}", link.server, encode(&link.deck))
}

/// Prints the URL, and opens it unless `--print` or `--json` says not to.
pub fn run(args: &OpenArgs) -> Outcome {
    let dir = args.dir.as_deref().unwrap_or(Path::new("."));
    let Some(link) = Link::load(dir)? else {
        let message =
            format!("{} is not linked to a deck — run `diapo push --create` or `diapo pull <id>`", dir.display());
        return Err(Failure::new("not_linked", message));
    };
    let url = web_url(&link);
    if ui::json() {
        print_json(&json!({ "url": url }));
        return Ok(());
    }
    println!("{url}");
    if !args.print {
        browser::open(&url);
    }
    Ok(())
}
