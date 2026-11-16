//! `diapo push` through the binary, against a mock server.

#[path = "support/deck.rs"]
mod deck;
#[path = "support/mock.rs"]
mod mock;

use std::fs;

use deck::{diapo, linked_folder, rev, text};
use diapo_cli::sync::{Link, sha256};
use mock::mock;

#[test]
fn push_stops_on_a_stale_revision_and_force_refetches_it() {
    let (url, seen) = mock(|line, body| match line {
        "PUT /api/decks/d1/source" if body.contains(r#""rev":5"#) => (200, r#"{"rev":6}"#.to_owned()),
        "PUT /api/decks/d1/source" => {
            (409, r#"{"error":{"code":"already_exists","message":"stale rev"},"rev":5}"#.to_owned())
        }
        "GET /api/decks/d1" => (200, r##"{"id":"d1","title":"T","source":"# Theirs\n","rev":5}"##.to_owned()),
        _ => (200, r#"{"assets":[]}"#.to_owned()),
    });
    let work = linked_folder(&url, &[]);
    let stale = diapo(&work, &["push", "talk", "--json"]);
    assert_eq!(stale.status.code(), Some(1));
    assert!(text(&stale.stderr).contains("`diapo pull`") && text(&stale.stderr).contains("`diapo push --force`"));
    let envelope: serde_json::Value = serde_json::from_slice(&stale.stdout).unwrap();
    assert_eq!(envelope["error"]["code"], "conflict");
    assert_eq!((rev(&work.path().join("talk")), seen.lock().unwrap().len()), (3, 1));
    let forced = diapo(&work, &["push", "talk", "--force"]);
    assert!(forced.status.success(), "{}", text(&forced.stderr));
    assert!(text(&forced.stdout).contains("Pushed talk/diapo.md to deck d1 (revision 6)"));
    assert_eq!(rev(&work.path().join("talk")), 6);
    assert!(seen.lock().unwrap().iter().all(|line| line.contains("[Bearer s3cret]")));
}

#[test]
fn push_syncs_assets_by_hash_and_leaves_what_it_never_had() {
    let listing = r#"{"assets":[{"name":"assets/kept.svg","sha256":"KEPT"},{"name":"assets/dropped.png","sha256":"h"},{"name":"assets/theirs.png","sha256":"z"}]}"#;
    let listing = listing.replace("KEPT", &sha256(b"<svg/>"));
    let (url, seen) = mock(move |line, _| match line {
        "PUT /api/decks/d1/source" => (200, r#"{"rev":4}"#.to_owned()),
        "GET /api/decks/d1/assets" => (200, listing.clone()),
        _ => (200, "{}".to_owned()),
    });
    let work = linked_folder(&url, &[("assets/dropped.png", "h")]);
    fs::write(work.path().join("talk/assets/kept.svg"), "<svg/>").unwrap();
    fs::write(work.path().join("talk/assets/new.svg"), "<svg>new</svg>").unwrap();
    let pushed = diapo(&work, &["push", "talk"]);
    let out = text(&pushed.stdout);
    assert!(pushed.status.success(), "{}", text(&pushed.stderr));
    assert!(out.contains("Uploading assets/new.svg") && out.contains("Deleting assets/dropped.png"), "{out}");
    assert!(out.contains("assets/theirs.png only on the server"), "{out}");
    let calls: Vec<String> =
        seen.lock().unwrap().iter().map(|line| line.split(" [").next().unwrap().to_owned()).collect();
    let expected = ["PUT /api/decks/d1/source", "GET /api/decks/d1/assets", "POST /api/decks/d1/assets"];
    assert_eq!(calls[..3], expected);
    assert_eq!(calls[3..], ["DELETE /api/decks/d1/assets/assets/dropped.png"]);
    let link = Link::load(&work.path().join("talk")).unwrap().unwrap();
    assert_eq!(link.assets.keys().collect::<Vec<_>>(), ["assets/kept.svg", "assets/new.svg"]);
}

const REJECTED: &str =
    r#"{"error":{"code":"invalid_argument","message":"line 1: x"},"errors":[{"line":1,"message":"x"}]}"#;

#[test]
fn push_creates_a_deck_from_the_folder_and_names_what_the_server_refuses() {
    let (url, seen) = mock(|line, body| match line {
        "POST /api/decks" => (201, r#"{"id":"d7","title":"Mine","rev":1}"#.to_owned()),
        "GET /api/decks/d7/assets" => (200, r#"{"assets":[]}"#.to_owned()),
        "POST /api/decks/d7/assets" if body.contains("notes.html") => {
            (400, r#"{"error":{"code":"invalid_argument","message":"this file type is not accepted"}}"#.to_owned())
        }
        "PUT /api/decks/d7/source" if body.contains("SERVER-REJECTS") => (422, REJECTED.to_owned()),
        "PUT /api/decks/d7/source" => (200, r#"{"rev":2}"#.to_owned()),
        _ => (201, "{}".to_owned()),
    });
    let work = linked_folder(&url, &[]);
    fs::remove_file(work.path().join("talk/.diapo.json")).unwrap();
    assert_eq!(diapo(&work, &["push", "talk"]).status.code(), Some(1));
    let created = diapo(&work, &["push", "talk", "--create"]);
    assert!(created.status.success(), "{}", text(&created.stderr));
    assert!(seen.lock().unwrap()[0].ends_with(r##"{"source":"# Mine\n","title":"talk"}"##));
    let link = Link::load(&work.path().join("talk")).unwrap().unwrap();
    assert_eq!((link.rev, link.deck.as_str()), (1, "d7"));
    fs::write(work.path().join("talk/diapo.md"), "# SERVER-REJECTS\n").unwrap();
    let invalid = diapo(&work, &["push", "talk"]);
    assert!(text(&invalid.stderr).contains("talk/diapo.md:1: x"), "{}", text(&invalid.stderr));
    fs::write(work.path().join("talk/assets/notes.html"), "<p>hi</p>").unwrap();
    fs::write(work.path().join("talk/diapo.md"), "# Mine\n\n![](assets/notes.html)\n").unwrap();
    let refused = diapo(&work, &["push", "talk"]);
    assert_eq!(refused.status.code(), Some(1));
    assert!(text(&refused.stderr).contains("the server refused assets/notes.html: this file type is not accepted"));
}
