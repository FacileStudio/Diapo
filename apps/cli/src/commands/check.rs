//! `diapo check`: every problem of a deck as `file:line: message`.

use serde_json::json;

use crate::cli::FolderArgs;
use crate::commands::print_json;
use crate::diagnose::{self, Report};
use crate::failure::{Failure, Outcome};
use crate::project::Folder;
use crate::ui;

fn summary(report: &Report) -> String {
    match &report.project {
        Some(project) => format!("{} scenes, {} steps", project.scenes().len(), project.steps()),
        None => String::new(),
    }
}

/// Prints the problems on stdout, which is what this command is for, and fails when there
/// is one.
pub fn run(args: &FolderArgs) -> Outcome {
    let folder = Folder::locate(args.dir.as_deref())?;
    let report = diagnose::inspect(&folder)?;
    let (label, valid) = (folder.label(), report.problems.is_empty());
    if ui::json() {
        print_json(&json!({ "file": label, "valid": valid, "errors": report.problems }));
    } else {
        diagnose::lines(&label, &report.problems).iter().for_each(|line| println!("{line}"));
    }
    if valid {
        ui::success(&format!("{label} is valid ({})", summary(&report)));
        return Ok(());
    }
    ui::error(&format!("{label} has {}", diagnose::count(report.problems.len())));
    Err(Failure::quiet("invalid"))
}
