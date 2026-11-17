//! The problems of a deck, each with its line and a stable code.

use std::cell::RefCell;

use scene::{ErrorKind, Options, Project};
use serde::Serialize;

use crate::failure::{Failure, Outcome};
use crate::project::Folder;

/// One problem in a source file.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    /// The 1-based line.
    pub line: usize,
    /// A stable identifier, such as `unknown_key` or `missing_asset`.
    pub code: String,
    /// The problem in words.
    pub message: String,
}

/// A source file read and checked.
pub struct Report {
    /// The source text.
    pub source: String,
    /// The project, when the source parses.
    pub project: Option<Project>,
    /// Every problem, in line order: parse errors, or the assets the folder lacks.
    pub problems: Vec<Diagnostic>,
}

/// The code of an error kind: its name in snake case, so a kind added to the format gets a
/// code with no change here.
pub fn code(kind: &ErrorKind) -> String {
    let debug = format!("{kind:?}");
    let name = debug.split(|c: char| !c.is_ascii_alphanumeric()).next().unwrap_or("invalid");
    let mut out = String::new();
    for letter in name.chars() {
        if letter.is_ascii_uppercase() && !out.is_empty() {
            out.push('_');
        }
        out.push(letter.to_ascii_lowercase());
    }
    out
}

fn missing_assets(folder: &Folder, source: &str, project: &Project) -> Vec<Diagnostic> {
    let missing = RefCell::new(Vec::new());
    let probe = |path: &str| -> Option<String> {
        if !folder.asset(path).is_file() {
            missing.borrow_mut().push(path.to_owned());
        }
        None
    };
    scene::compile(project, &Options::new().assets(&probe));
    let mut problems: Vec<Diagnostic> = missing.into_inner().into_iter().map(|path| lost(source, &path)).collect();
    problems.sort_by_key(|problem| problem.line);
    problems.dedup();
    problems
}

fn lost(source: &str, path: &str) -> Diagnostic {
    let line = source.lines().position(|line| line.contains(path)).map_or(1, |index| index + 1);
    Diagnostic { line, code: "missing_asset".to_owned(), message: format!("asset `{path}` is not in the deck folder") }
}

/// Reads and checks the deck of `folder`.
pub fn inspect(folder: &Folder) -> Outcome<Report> {
    let source = folder.source()?;
    Ok(match scene::planche::project(&source) {
        Ok(project) => {
            let problems = missing_assets(folder, &source, &project);
            Report { source, project: Some(project), problems }
        }
        Err(errors) => {
            let problems = errors.0.iter().map(|error| Diagnostic {
                line: error.line,
                code: code(&error.kind),
                message: error.kind.to_string(),
            });
            Report { problems: problems.collect(), source, project: None }
        }
    })
}

/// Each problem as `file:line: message`.
pub fn lines(label: &str, problems: &[Diagnostic]) -> Vec<String> {
    problems.iter().map(|problem| format!("{label}:{}: {}", problem.line, problem.message)).collect()
}

/// The source and the project of a deck with no problem. Otherwise prints every problem on
/// stderr and fails.
pub fn require(folder: &Folder) -> Outcome<(String, Project)> {
    let report = inspect(folder)?;
    match report.project.filter(|_| report.problems.is_empty()) {
        Some(project) => Ok((report.source, project)),
        None => {
            lines(&folder.label(), &report.problems).iter().for_each(|line| eprintln!("{line}"));
            Err(Failure::new("invalid", format!("{} has {}", folder.label(), count(report.problems.len()))))
        }
    }
}

/// `1 problem`, `3 problems`.
pub fn count(problems: usize) -> String {
    if problems == 1 { "1 problem".to_owned() } else { format!("{problems} problems") }
}
