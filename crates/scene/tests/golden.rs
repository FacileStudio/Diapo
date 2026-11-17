//! Golden tests: each fixture compiles to the checked-in document, and each invalid fixture
//! reports the checked-in problems. `UPDATE_GOLDEN=1 cargo test -p facile-scene` rewrites them.

mod support;

use std::path::{Path, PathBuf};

use support::{body_of, golden};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures")
}

#[test]
fn every_fixture_compiles_to_its_golden_document() {
    for (folder, entry) in [("basic", "diapo.md"), ("steps", "diapo.md"), ("raw", "clap.md")] {
        let dir = fixtures().join(folder);
        let project = scene::parse(&std::fs::read_to_string(dir.join(entry)).unwrap()).unwrap();
        let html = scene::compile(&project, &scene::Options::new());
        golden(&dir.join("expected.html"), &body_of(&html));
    }
}

#[test]
fn invalid_sources_report_their_golden_problems() {
    for name in ["frontmatter", "steps", "remote", "unclosed", "empty"] {
        let dir = fixtures().join("invalid");
        let source = std::fs::read_to_string(dir.join(format!("{name}.md"))).unwrap();
        let errors = scene::parse(&source).expect_err(name);
        golden(&dir.join(format!("{name}.errors")), &format!("{errors}\n"));
    }
}
