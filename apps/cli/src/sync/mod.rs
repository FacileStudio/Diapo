//! What links a folder to a deck on a server: the `.diapo.json` file, and the content hashes
//! `pull` and `push` compare.

pub mod plan;

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::failure::{Failure, Outcome};

/// The file that links a folder to its deck.
pub const LINK_FILE: &str = ".diapo.json";

/// The folder, beside the entry file, whose files are the deck's assets.
pub const ASSETS_DIR: &str = "assets";

/// Asset names to the SHA-256 of their content. A name is the path the source writes,
/// relative to the deck folder, such as `assets/chart.png`.
pub type Hashes = BTreeMap<String, String>;

/// The content of `.diapo.json`: the deck, and what it held at the last pull or push.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// The deck id.
    pub deck: String,
    /// The site root of the server that holds the deck.
    pub server: String,
    /// The revision of the source at the last pull or push.
    pub rev: i64,
    /// The SHA-256 of the source at the last pull or push.
    #[serde(default)]
    pub source_sha256: String,
    /// The assets at the last pull or push.
    #[serde(default)]
    pub assets: Hashes,
}

impl Link {
    /// Reads the link of `dir`, or `None` when the folder is not linked.
    pub fn load(dir: &Path) -> Outcome<Option<Link>> {
        let path = dir.join(LINK_FILE);
        if !path.is_file() {
            return Ok(None);
        }
        let reason = |detail: String| Failure::new("bad_link", format!("cannot read {}: {detail}", path.display()));
        let text = fs::read_to_string(&path).map_err(|error| reason(error.to_string()))?;
        serde_json::from_str(&text).map(Some).map_err(|error| reason(error.to_string()))
    }

    /// Writes the link into `dir`.
    pub fn save(&self, dir: &Path) -> Outcome {
        let text = serde_json::to_string_pretty(self).map_err(io::Error::other)?;
        Ok(fs::write(dir.join(LINK_FILE), text + "\n")?)
    }
}

/// The SHA-256 of `bytes`, in lowercase hex.
pub fn sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/// Whether an asset name may be written into a deck folder: a relative path with no empty,
/// `.` or `..` part, no hidden part, and not the deck's own source file. The server's names
/// are checked with this before a pull writes anything.
pub fn safe_name(name: &str) -> bool {
    let plain =
        |part: Component<'_>| matches!(part, Component::Normal(text) if !text.to_string_lossy().starts_with('.'));
    !name.is_empty() && name != crate::project::ENTRY && Path::new(name).components().all(plain)
}

/// The file an asset name is in `dir`.
pub fn asset_path(dir: &Path, name: &str) -> PathBuf {
    dir.join(name)
}

fn walk(root: &Path, at: &Path, found: &mut Hashes) -> io::Result<()> {
    for entry in fs::read_dir(at)? {
        let path = entry?.path();
        let hidden = path.file_name().is_some_and(|name| name.to_string_lossy().starts_with('.'));
        if hidden {
            continue;
        }
        if path.is_dir() {
            walk(root, &path, found)?;
        } else if let Ok(relative) = path.strip_prefix(root) {
            let name: Vec<String> = relative.iter().map(|part| part.to_string_lossy().into_owned()).collect();
            found.insert(name.join("/"), sha256(&fs::read(&path)?));
        }
    }
    Ok(())
}

/// Hashes the folder's assets: every file under `dir/assets`, hidden files left out, and
/// each file of `known`, the last sync, that still exists elsewhere in the folder.
pub fn local_assets(dir: &Path, known: &Hashes) -> io::Result<Hashes> {
    let mut found = Hashes::new();
    if dir.join(ASSETS_DIR).is_dir() {
        walk(dir, &dir.join(ASSETS_DIR), &mut found)?;
    }
    for name in known.keys().filter(|name| safe_name(name)) {
        if let Ok(bytes) = fs::read(asset_path(dir, name)) {
            found.insert(name.clone(), sha256(&bytes));
        }
    }
    Ok(found)
}
