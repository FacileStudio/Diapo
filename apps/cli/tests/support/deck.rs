//! A linked deck folder and a stored login, for the sync commands.

#[path = "bin.rs"]
mod bin;

use std::fs;
use std::path::Path;
use std::process::Output;

use diapo_cli::sync::Link;
use tempfile::TempDir;

/// Runs `diapo` in `work`, which is also its config home.
pub fn diapo(work: &TempDir, args: &[&str]) -> Output {
    bin::command(work.path(), work.path(), args).output().unwrap()
}

/// Bytes as text.
pub fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// A logged-in config and a `talk` folder linked to deck `d1` at revision 3, with `base` as
/// the assets of its last sync.
pub fn linked_folder(server: &str, base: &[(&str, &str)]) -> TempDir {
    let work = tempfile::tempdir().unwrap();
    fs::create_dir_all(work.path().join("diapo")).unwrap();
    fs::write(work.path().join("diapo/config.yml"), format!("server_url: {server}/api\ntoken: s3cret\n")).unwrap();
    fs::create_dir_all(work.path().join("talk/assets")).unwrap();
    fs::write(work.path().join("talk/diapo.md"), "# Mine\n").unwrap();
    let assets = base.iter().map(|(name, hash)| ((*name).to_owned(), (*hash).to_owned())).collect();
    let link = Link { deck: "d1".to_owned(), server: server.to_owned(), rev: 3, assets, ..Link::default() };
    link.save(&work.path().join("talk")).unwrap();
    work
}

/// The revision the folder's link holds.
pub fn rev(dir: &Path) -> i64 {
    Link::load(dir).unwrap().unwrap().rev
}
