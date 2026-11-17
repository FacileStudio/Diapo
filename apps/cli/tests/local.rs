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

const PROJECTION: &str = "---\nplanche: 1\nsize: 1920x1080\n---\n\n\
````frame id=\"one\" x=\"0\" y=\"0\" w=\"1920\" h=\"1080\" name=\"Talk\" order=\"a0\"\n\
```text id=\"title\" x=\"160\" y=\"120\" w=\"1600\" h=\"200\" z=\"a0\"\n\
# Talk\n\
```\n\n\
```image id=\"chart\" x=\"1100\" y=\"400\" w=\"660\" h=\"500\" z=\"a1\" asset=\"assets/chart.svg\"\n\
```\n\
````\n";

const LEGACY: &str = "---\ntitle: Talk\n---\n\n# Talk\n\n![](assets/chart.svg)\n\n---\n\n## Second\n";

fn folder(work: &Path, source: &str) {
    fs::create_dir_all(work.join("talk/assets")).unwrap();
    fs::write(work.join("talk/diapo.md"), source).unwrap();
    fs::write(work.join("talk/assets/chart.svg"), "<svg xmlns=\"http://www.w3.org/2000/svg\"/>").unwrap();
}

#[test]
fn new_writes_a_planche_starter_that_checks_and_builds_clean() {
    let work = tempfile::tempdir().unwrap();
    assert!(diapo(work.path(), work.path(), &["new", "talk"]).status.success());
    let source = fs::read_to_string(work.path().join("talk/diapo.md")).unwrap();
    assert_eq!(source, scene::planche::STARTER.replace("New deck", "talk"));
    assert!(work.path().join("talk/assets").is_dir());
    let checked = diapo(work.path(), work.path(), &["check", "talk"]);
    assert!(
        checked.status.success() && text(&checked.stdout).contains("is valid (1 scene"),
        "{}",
        text(&checked.stdout)
    );
    let built = diapo(work.path(), work.path(), &["build", "talk"]);
    assert!(built.status.success(), "{}", text(&built.stderr));
    let html = fs::read_to_string(work.path().join("talk.html")).unwrap();
    assert!(html.contains("<title>talk</title>") && html.contains("pl-text"), "{html}");
    assert_eq!(diapo(work.path(), work.path(), &["new", "talk"]).status.code(), Some(1));
}

#[test]
fn a_projection_and_a_legacy_folder_both_check_and_build_into_one_file() {
    for (source, scenes) in [(PROJECTION, "1 scene"), (LEGACY, "2 scenes")] {
        let work = tempfile::tempdir().unwrap();
        folder(work.path(), source);
        let checked = diapo(work.path(), work.path(), &["check", "talk"]);
        assert!(checked.status.success(), "{}", text(&checked.stdout));
        assert!(text(&checked.stdout).contains(&format!("is valid ({scenes}")), "{}", text(&checked.stdout));
        let built = diapo(work.path(), work.path(), &["build", "talk"]);
        assert!(built.status.success(), "{}", text(&built.stderr));
        let html = fs::read_to_string(work.path().join("talk.html")).unwrap();
        assert!(html.contains("<title>Talk</title>") && html.contains("data:image/svg+xml;base64,"));
        for external in ["src=\"//", "src=\"assets/", "url(assets/"] {
            assert!(!html.contains(external), "the built file still names {external}");
        }
    }
}

#[test]
fn build_refuses_a_missing_asset_and_check_names_projection_lines() {
    let work = tempfile::tempdir().unwrap();
    folder(work.path(), PROJECTION);
    fs::remove_file(work.path().join("talk/assets/chart.svg")).unwrap();
    let built = diapo(work.path(), work.path(), &["build", "talk", "-o", "out.html"]);
    assert_eq!(built.status.code(), Some(1));
    assert!(text(&built.stderr).contains("talk/diapo.md:11: asset `assets/chart.svg` is not in the deck folder"));
    assert!(!work.path().join("out.html").exists());
    fs::write(work.path().join("talk/diapo.md"), PROJECTION.replace("```image", "```picture")).unwrap();
    let checked = diapo(work.path(), work.path(), &["check", "talk", "--json"]);
    let answer: serde_json::Value = serde_json::from_slice(&checked.stdout).unwrap();
    assert_eq!((checked.status.code(), &answer["errors"][0]["line"]), (Some(1), &serde_json::json!(11)));
    assert_eq!(answer["errors"][0]["code"], "projection");
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
    let guide = text(&guide.stdout);
    assert!(
        guide.starts_with("# Decks as planche documents") && guide.contains("older scene format is still accepted")
    );
    let export = diapo(work.path(), work.path(), &["export", "talk", "--format", "pdf"]);
    assert_eq!(export.status.code(), Some(1));
    assert!(text(&export.stderr).contains("`diapo build`") && export.stdout.is_empty());
    let nothing = diapo(work.path(), work.path(), &["check"]);
    assert!(text(&nothing.stderr).contains("no diapo.md in ."), "{}", text(&nothing.stderr));
}
