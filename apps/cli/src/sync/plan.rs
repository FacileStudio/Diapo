//! The decisions of `pull` and `push`, as pure functions of three states: `base` (what both
//! sides held at the last sync, from `.diapo.json`), `local` (the folder now) and `remote`
//! (the server now).

use super::Hashes;

/// What `push` does to the server's assets.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PushPlan {
    /// Local files the server lacks or holds with other content.
    pub upload: Vec<String>,
    /// Server files that were synced before and are gone from the folder.
    pub delete: Vec<String>,
    /// Server files the folder never had: left alone, and reported.
    pub remote_only: Vec<String>,
}

/// What `pull` does to the folder's assets.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PullPlan {
    /// Server files the folder lacks or holds with other content.
    pub download: Vec<String>,
    /// Local files that were synced before, are unchanged, and are gone from the server.
    pub remove: Vec<String>,
    /// Files `download` would overwrite although they changed locally since the last sync.
    pub conflicts: Vec<String>,
}

/// What `pull` does to the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourcePull {
    /// The folder already holds the server's source.
    UpToDate,
    /// The folder holds the last synced source, or none: take the server's.
    Take,
    /// The folder holds edits the server does not have: taking the server's would lose them.
    LocalChanges,
}

/// Plans the asset half of a push.
pub fn push(base: &Hashes, local: &Hashes, remote: &Hashes) -> PushPlan {
    let mut plan = PushPlan::default();
    for (name, hash) in local {
        if remote.get(name) != Some(hash) {
            plan.upload.push(name.clone());
        }
    }
    for name in remote.keys().filter(|name| !local.contains_key(*name)) {
        match base.contains_key(name) {
            true => plan.delete.push(name.clone()),
            false => plan.remote_only.push(name.clone()),
        }
    }
    plan
}

/// Plans the asset half of a pull.
pub fn pull(base: &Hashes, local: &Hashes, remote: &Hashes) -> PullPlan {
    let mut plan = PullPlan::default();
    for (name, hash) in remote {
        let here = local.get(name);
        if here == Some(hash) {
            continue;
        }
        if here.is_some() && here != base.get(name) {
            plan.conflicts.push(name.clone());
        }
        plan.download.push(name.clone());
    }
    for (name, hash) in base {
        if !remote.contains_key(name) && local.get(name) == Some(hash) {
            plan.remove.push(name.clone());
        }
    }
    plan
}

/// Decides the source half of a pull from three hashes: the last synced source, the folder's
/// source when it has one, and the server's.
pub fn pull_source(base: &str, local: Option<&str>, remote: &str) -> SourcePull {
    match local {
        Some(local) if local == remote => SourcePull::UpToDate,
        Some(local) if local != base => SourcePull::LocalChanges,
        _ => SourcePull::Take,
    }
}
