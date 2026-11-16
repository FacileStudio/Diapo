//! `diapo pull` through the binary, against a mock server.

#[path = "support/deck.rs"]
mod deck;
#[path = "support/mock.rs"]
mod mock;

use std::fs;

use deck::{diapo, linked_folder, rev, text};
use diapo_cli::sync::sha256;
use mock::mock;

fn library(line: &str, _: &str) -> (u16, String) {
    let listing = format!(r#"{{"assets":[{{"name":"assets/chart.svg","size":6,"sha256":"{}"}}]}}"#, sha256(b"<svg/>"));
    match line {
        "GET /api/decks/d9" => (200, r##"{"id":"d9","title":"Q3 Review","source":"# Remote\n","rev":7}"##.to_owned()),
        "GET /api/decks/d9/assets" => (200, listing),
        "GET /api/decks/d9/assets/assets/chart.svg" => (200, "<svg/>".to_owned()),
        "GET /api/decks/evil/assets" => (200, r#"[{"name":"../../escaped.txt","sha256":"x"}]"#.to_owned()),
        "GET /api/decks/evil" => (200, r##"{"id":"evil","title":"Evil","source":"# x\n","rev":1}"##.to_owned()),
        _ => (404, r#"{"error":{"code":"not_found","message":"deck not found"}}"#.to_owned()),
    }
}

#[test]
fn pull_writes_the_deck_and_refuses_to_lose_local_edits() {
    let (url, _) = mock(library);
    let work = linked_folder(&url, &[]);
    let pulled = diapo(&work, &["pull", &format!("{url}/d/d9")]);
    assert!(pulled.status.success(), "{}", text(&pulled.stderr));
    let dir = work.path().join("q3-review");
    assert_eq!(fs::read_to_string(dir.join("diapo.md")).unwrap(), "# Remote\n");
    assert_eq!(fs::read_to_string(dir.join("assets/chart.svg")).unwrap(), "<svg/>");
    assert_eq!(rev(&dir), 7);
    assert!(text(&diapo(&work, &["pull", "q3-review"]).stdout).contains("q3-review is up to date (revision 7)"));
    fs::write(dir.join("diapo.md"), "# Edited here\n").unwrap();
    let refused = diapo(&work, &["pull", "q3-review"]);
    assert_eq!(refused.status.code(), Some(1));
    assert!(text(&refused.stderr).contains("diapo.md changed here since the last sync"), "{}", text(&refused.stderr));
    assert_eq!(fs::read_to_string(dir.join("diapo.md")).unwrap(), "# Edited here\n");
    assert!(diapo(&work, &["pull", "q3-review", "--force"]).status.success());
    assert_eq!(fs::read_to_string(dir.join("diapo.md")).unwrap(), "# Remote\n");
}

#[test]
fn pull_refuses_an_asset_name_that_leaves_the_folder_and_reports_a_missing_deck() {
    let (url, _) = mock(library);
    let work = linked_folder(&url, &[]);
    let evil = diapo(&work, &["pull", "evil", "inside/deck"]);
    assert_eq!(evil.status.code(), Some(1));
    assert!(text(&evil.stderr).contains("unsafe name"), "{}", text(&evil.stderr));
    assert!(!work.path().join("escaped.txt").exists() && !work.path().join("inside").exists());
    let missing = diapo(&work, &["pull", "nope"]);
    assert_eq!(missing.status.code(), Some(1));
    assert!(text(&missing.stderr).contains("the server answered 404: deck not found"), "{}", text(&missing.stderr));
}
