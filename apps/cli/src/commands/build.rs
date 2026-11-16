//! `diapo build`: one offline HTML file, every asset inlined as a `data:` URI.

use std::fs;
use std::path::PathBuf;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use scene::Options;
use serde_json::json;

use crate::cli::BuildArgs;
use crate::commands::print_json;
use crate::diagnose;
use crate::failure::Outcome;
use crate::mime;
use crate::project::Folder;
use crate::ui;

/// The compiled document of `folder`, with each of its files inlined.
pub fn document(folder: &Folder) -> Outcome<(String, usize)> {
    let (_, project) = diagnose::require(folder)?;
    let inline = |path: &str| -> Option<String> {
        let file = folder.asset(path);
        let bytes = fs::read(&file).ok()?;
        Some(format!("data:{};base64,{}", mime::of(&file).replace("; ", ";"), STANDARD.encode(bytes)))
    };
    Ok((scene::compile(&project, &Options::new().assets(&inline)), project.scenes().len()))
}

/// Writes the document to `--output`, or to `<folder name>.html` in the current folder.
pub fn run(args: &BuildArgs) -> Outcome {
    let folder = Folder::locate(args.dir.as_deref())?;
    let (html, scenes) = document(&folder)?;
    let output = args.output.clone().unwrap_or_else(|| PathBuf::from(format!("{}.html", folder.name())));
    fs::write(&output, &html)?;
    if ui::json() {
        print_json(&json!({ "output": output, "scenes": scenes, "bytes": html.len() }));
        return Ok(());
    }
    ui::success(&format!("Wrote {} ({scenes} scenes, {} kB)", output.display(), html.len().div_ceil(1000)));
    Ok(())
}
