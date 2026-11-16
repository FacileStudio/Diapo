//! `diapo new`: a folder with a starter deck.

use std::fs;

use serde_json::json;

use crate::cli::NewArgs;
use crate::commands::print_json;
use crate::failure::{Failure, Outcome};
use crate::project::ENTRY;
use crate::sync::ASSETS_DIR;
use crate::ui;

/// The starter deck. `{{title}}` is replaced by the folder's name.
pub const STARTER: &str = include_str!("../../starter/diapo.md");

const CHART: &str = include_str!("../../starter/assets/chart.svg");

/// The starter deck titled `title`.
pub fn starter(title: &str) -> String {
    let safe: String = title.chars().filter(|letter| !letter.is_control() && *letter != '"').collect();
    STARTER.replace("title: {{title}}", &format!("title: \"{safe}\"")).replace("{{title}}", &safe)
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
    fs::write(dir.join(ASSETS_DIR).join("chart.svg"), CHART)?;
    if ui::json() {
        print_json(&json!({ "dir": dir, "entry": dir.join(ENTRY) }));
        return Ok(());
    }
    ui::success(&format!("Created {shown} with {ENTRY} and {ASSETS_DIR}/"));
    ui::hint(&format!("Run `diapo serve {shown}` to see it"));
    Ok(())
}
