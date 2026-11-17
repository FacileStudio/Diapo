//! `diapo new`: a folder with a starter deck.

use std::fs;

use serde_json::json;

use crate::cli::NewArgs;
use crate::commands::print_json;
use crate::failure::{Failure, Outcome};
use crate::project::ENTRY;
use crate::sync::ASSETS_DIR;
use crate::ui;

const PLACEHOLDER: &str = "New deck";

/// The planche starter deck, `scene::planche::STARTER`, titled `title`.
pub fn starter(title: &str) -> String {
    let safe: String =
        title.chars().filter(|letter| !letter.is_control() && !matches!(letter, '"' | '\\' | '`')).collect();
    match safe.trim() {
        "" => scene::planche::STARTER.to_owned(),
        safe => scene::planche::STARTER.replace(PLACEHOLDER, safe),
    }
}

/// Creates the folder, its `diapo.md` and its `assets/`.
pub fn run(args: &NewArgs) -> Outcome {
    let dir = &args.name;
    let shown = dir.display();
    if dir.exists() {
        return Err(Failure::new("exists", format!("{shown} already exists — pick another name")));
    }
    let title = dir.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    fs::create_dir_all(dir.join(ASSETS_DIR))?;
    fs::write(dir.join(ENTRY), starter(&title))?;
    if ui::json() {
        print_json(&json!({ "dir": dir, "entry": dir.join(ENTRY) }));
        return Ok(());
    }
    ui::success(&format!("Created {shown} with {ENTRY} and {ASSETS_DIR}/"));
    ui::hint(&format!("Run `diapo serve {shown}` to see it"));
    Ok(())
}
