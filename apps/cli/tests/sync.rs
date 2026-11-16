//! The sync state machine: what `pull` and `push` decide from the last synced state, the
//! folder and the server.

use std::fs;

use diapo_cli::sync::plan::{self, SourcePull};
use diapo_cli::sync::{Hashes, Link, asset_name, local_assets, safe_name, sha256};

fn hashes(pairs: &[(&str, &str)]) -> Hashes {
    pairs.iter().map(|(name, hash)| ((*name).to_owned(), (*hash).to_owned())).collect()
}

#[test]
fn push_uploads_what_differs_and_deletes_only_what_it_once_synced() {
    let base = hashes(&[("same.png", "1"), ("edited.png", "2"), ("dropped.png", "3")]);
    let local = hashes(&[("same.png", "1"), ("edited.png", "2b"), ("new.png", "4")]);
    let remote = hashes(&[("same.png", "1"), ("edited.png", "2"), ("dropped.png", "3"), ("theirs.png", "5")]);
    let plan = plan::push(&base, &local, &remote);
    assert_eq!(plan.upload, ["edited.png", "new.png"]);
    assert_eq!(plan.delete, ["dropped.png"]);
    assert_eq!(plan.remote_only, ["theirs.png"]);
    assert_eq!(plan::push(&local, &local, &local), plan::PushPlan::default());
}

#[test]
fn pull_downloads_what_differs_and_flags_what_changed_on_both_sides() {
    let base = hashes(&[("same.png", "1"), ("theirs.png", "2"), ("both.png", "3")]);
    let local = hashes(&[("same.png", "1"), ("theirs.png", "2"), ("both.png", "3-mine"), ("mine.png", "9")]);
    let remote = hashes(&[("same.png", "1"), ("theirs.png", "2-new"), ("both.png", "3-theirs"), ("added.png", "4")]);
    let plan = plan::pull(&base, &local, &remote);
    assert_eq!(plan.download, ["added.png", "both.png", "theirs.png"]);
    assert_eq!(plan.conflicts, ["both.png"]);
    assert!(plan.remove.is_empty());
}

#[test]
fn pull_removes_a_file_the_server_dropped_unless_it_was_edited_here() {
    let base = hashes(&[("gone.png", "1"), ("gone-but-edited.png", "2"), ("already-gone.png", "3")]);
    let local = hashes(&[("gone.png", "1"), ("gone-but-edited.png", "2-mine"), ("untracked.png", "7")]);
    let plan = plan::pull(&base, &local, &Hashes::new());
    assert_eq!(plan.remove, ["gone.png"]);
    assert!(plan.download.is_empty() && plan.conflicts.is_empty());
}

#[test]
fn pull_takes_the_source_unless_the_folder_holds_unsynced_edits() {
    assert_eq!(plan::pull_source("", None, "remote"), SourcePull::Take);
    assert_eq!(plan::pull_source("base", Some("base"), "remote"), SourcePull::Take);
    assert_eq!(plan::pull_source("base", Some("remote"), "remote"), SourcePull::UpToDate);
    assert_eq!(plan::pull_source("base", Some("mine"), "remote"), SourcePull::LocalChanges);
    assert_eq!(plan::pull_source("", Some("mine"), "remote"), SourcePull::LocalChanges);
}

#[test]
fn the_link_file_round_trips_and_assets_are_hashed_without_hidden_files() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(Link::load(dir.path()).unwrap(), None);
    fs::create_dir_all(dir.path().join("assets/img")).unwrap();
    fs::write(dir.path().join("assets/img/a.png"), b"abc").unwrap();
    fs::write(dir.path().join("assets/.DS_Store"), b"x").unwrap();
    let assets = local_assets(dir.path()).unwrap();
    let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    assert_eq!(assets, hashes(&[("img/a.png", abc)]));
    assert_eq!(sha256(b"abc"), abc);
    let link =
        Link { deck: "d1".to_owned(), server: "https://diapo.example".to_owned(), rev: 3, assets, ..Link::default() };
    link.save(dir.path()).unwrap();
    assert_eq!(Link::load(dir.path()).unwrap(), Some(link));
}

#[test]
fn asset_names_from_a_server_cannot_leave_the_assets_folder() {
    assert_eq!(asset_name("assets/chart.svg"), "chart.svg");
    assert_eq!(asset_name("chart.svg"), "chart.svg");
    assert!(safe_name("img/chart.svg"));
    for name in ["", "../x", "img/../../x", "/etc/passwd", "./x"] {
        assert!(!safe_name(name), "{name}");
    }
}
