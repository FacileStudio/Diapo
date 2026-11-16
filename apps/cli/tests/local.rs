//! The offline commands, through the binary.

#[path = "support/bin.rs"]
mod bin;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../crates/scene/fixtures")
}

fn diapo(cwd: &Path, home: &Path, args: &[&str]) -> Output {
    bin::command(cwd, home, args).output().unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn new_then_build_writes_one_file_that_loads_nothing_else() {
    let work = tempfile::tempdir().unwrap();
    assert!(diapo(work.path(), work.path(), &["new", "talk"]).status.success());
    assert!(work.path().join("talk/diapo.md").is_file() && work.path().join("talk/assets/chart.svg").is_file());
    let built = diapo(work.path(), work.path(), &["build", "talk"]);
    assert!(built.status.success(), "{}", text(&built.stderr));
    let html = fs::read_to_string(work.path().join("talk.html")).unwrap();
    assert!(html.contains("<title>talk</title>") && html.contains("data:image/svg+xml;base64,"));
    for external in ["http://", "https://", "src=\"//", "src=\"assets/", "url(assets/"] {
        assert!(!html.contains(external), "the built file still names {external}");
    }
    let again = diapo(work.path(), work.path(), &["new", "talk"]);
    assert_eq!(again.status.code(), Some(1));
}

#[test]
fn build_refuses_a_deck_whose_asset_is_missing() {
    let work = tempfile::tempdir().unwrap();
    diapo(work.path(), work.path(), &["new", "talk"]);
    fs::remove_file(work.path().join("talk/assets/chart.svg")).unwrap();
    let built = diapo(work.path(), work.path(), &["build", "talk", "-o", "out.html"]);
    assert_eq!(built.status.code(), Some(1));
    assert!(text(&built.stderr).contains("talk/diapo.md:30: asset `assets/chart.svg` is not in the deck folder"));
    assert!(!work.path().join("out.html").exists());
}

#[test]
fn check_prints_every_error_of_the_invalid_fixtures_with_file_and_line() {
    let work = tempfile::tempdir().unwrap();
    for name in ["empty", "frontmatter", "remote", "steps", "unclosed"] {
        let file = fixtures().join(format!("invalid/{name}.md"));
        let shown = file.display().to_string();
        let checked = diapo(work.path(), work.path(), &["check", &shown]);
        assert_eq!(checked.status.code(), Some(1), "{name}");
        let expected = fs::read_to_string(fixtures().join(format!("invalid/{name}.errors"))).unwrap();
        let expected: Vec<String> =
            expected.lines().map(|line| line.replacen("line ", &format!("{shown}:"), 1)).collect();
        assert_eq!(text(&checked.stdout).lines().collect::<Vec<_>>(), expected, "{name}");
    }
}

#[test]
fn check_accepts_a_valid_deck_and_speaks_json() {
    let work = tempfile::tempdir().unwrap();
    let basic = fixtures().join("basic").display().to_string();
    let valid = diapo(work.path(), work.path(), &["check", &basic]);
    assert!(valid.status.success() && text(&valid.stdout).contains("is valid (5 scenes"), "{}", text(&valid.stdout));
    let steps = fixtures().join("invalid/steps.md").display().to_string();
    let invalid = diapo(work.path(), work.path(), &["check", "--json", &steps]);
    let answer: serde_json::Value = serde_json::from_slice(&invalid.stdout).unwrap();
    assert_eq!(invalid.status.code(), Some(1));
    assert_eq!(answer["valid"], false);
    assert_eq!(answer["errors"][0]["line"], 4);
    assert_eq!(answer["errors"][0]["code"], "step_out_of_range");
}

#[test]
fn version_usage_errors_guide_and_export_follow_the_standard() {
    let work = tempfile::tempdir().unwrap();
    let version = diapo(work.path(), work.path(), &["--version"]);
    assert_eq!(text(&version.stdout), format!("diapo {}\n", env!("CARGO_PKG_VERSION")));
    assert_eq!(diapo(work.path(), work.path(), &["frobnicate"]).status.code(), Some(2));
    let guide = diapo(work.path(), work.path(), &["guide"]);
    assert!(text(&guide.stdout).starts_with("# The scene format"));
    let export = diapo(work.path(), work.path(), &["export", "talk", "--format", "pdf"]);
    assert_eq!(export.status.code(), Some(1));
    assert!(text(&export.stderr).contains("`diapo build`") && export.stdout.is_empty());
    let nothing = diapo(work.path(), work.path(), &["check"]);
    assert!(text(&nothing.stderr).contains("no diapo.md in ."), "{}", text(&nothing.stderr));
}
