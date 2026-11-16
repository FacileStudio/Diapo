//! A deck on disk: a folder, its entry file and the files beside it.

use std::fs;
use std::path::{Path, PathBuf};

use crate::failure::{Failure, Outcome};
use crate::url::decode;

/// The entry file of a deck folder.
pub const ENTRY: &str = "diapo.md";

/// A deck folder and its entry file.
#[derive(Debug, Clone, PartialEq)]
pub struct Folder {
    /// The folder the assets are read from.
    pub dir: PathBuf,
    /// The source file.
    pub entry: PathBuf,
}

impl Folder {
    /// Finds the deck at `path`: a folder holding `diapo.md`, or any source file, whose
    /// folder is then the deck folder. Without a path, the current folder.
    pub fn locate(path: Option<&Path>) -> Outcome<Folder> {
        let path = path.unwrap_or(Path::new("."));
        if path.is_file() {
            let dir = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."));
            return Ok(Folder { dir: dir.to_owned(), entry: path.to_owned() });
        }
        let entry = path.join(ENTRY);
        if !entry.is_file() {
            let shown = path.display();
            return Err(Failure::new(
                "no_deck",
                format!("no {ENTRY} in {shown} — run `diapo new <name>` to start a deck"),
            ));
        }
        Ok(Folder { dir: path.to_owned(), entry })
    }

    /// The source text.
    pub fn source(&self) -> Outcome<String> {
        fs::read_to_string(&self.entry)
            .map_err(|error| Failure::new("io", format!("cannot read {}: {error}", self.label())))
    }

    /// The entry file as messages name it.
    pub fn label(&self) -> String {
        let shown = self.entry.display().to_string();
        shown.strip_prefix("./").unwrap_or(&shown).to_owned()
    }

    /// The folder's own name, which names the built file and a new deck.
    pub fn name(&self) -> String {
        let named = |path: &Path| path.file_name().map(|name| name.to_string_lossy().into_owned());
        named(&self.dir)
            .or_else(|| fs::canonicalize(&self.dir).ok().and_then(|path| named(&path)))
            .unwrap_or("deck".to_owned())
    }

    /// The file a source path such as `assets/chart.png` names.
    pub fn asset(&self, path: &str) -> PathBuf {
        let bare = path.split(['?', '#']).next().unwrap_or(path);
        self.dir.join(decode(bare))
    }
}
