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
        "PUT /api/decks/d1/source" => (409, r#"{"error":{"code":"conflict","message":"stale rev"}}"#.to_owned()),
        "GET /api/decks/d1" => (200, r##"{"id":"d1","title":"T","source":"# Theirs\n","rev":5}"##.to_owned()),
        _ => (200, "[]".to_owned()),
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
    let listing = r#"[{"name":"kept.svg","sha256":"KEPT"},{"name":"dropped.png","sha256":"h"},{"name":"theirs.png","sha256":"z"}]"#;
    let listing = listing.replace("KEPT", &sha256(b"<svg/>"));
    let (url, seen) = mock(move |line, _| match line {
        "PUT /api/decks/d1/source" => (200, r#"{"rev":4}"#.to_owned()),
        "GET /api/decks/d1/assets" => (200, listing.clone()),
        _ => (200, "{}".to_owned()),
    });
    let work = linked_folder(&url, &[("dropped.png", "h")]);
    fs::write(work.path().join("talk/assets/kept.svg"), "<svg/>").unwrap();
    fs::write(work.path().join("talk/assets/new.svg"), "<svg>new</svg>").unwrap();
    let pushed = diapo(&work, &["push", "talk"]);
    let out = text(&pushed.stdout);
    assert!(pushed.status.success(), "{}", text(&pushed.stderr));
    assert!(
        out.contains("\u{2191} new.svg")
            && out.contains("- dropped.png")
            && out.contains("theirs.png only on the server")
    );
    let calls: Vec<String> =
        seen.lock().unwrap().iter().map(|line| line.split(" [").next().unwrap().to_owned()).collect();
    let expected = ["PUT /api/decks/d1/source", "GET /api/decks/d1/assets", "POST /api/decks/d1/assets"];
    assert_eq!(calls[..3], expected);
    assert_eq!(calls[3..], ["DELETE /api/decks/d1/assets/dropped.png"]);
    let link = Link::load(&work.path().join("talk")).unwrap().unwrap();
    assert_eq!(link.assets.keys().collect::<Vec<_>>(), ["kept.svg", "new.svg"]);
}
