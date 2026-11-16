//! Reading and writing the credential file with the right permissions.

use std::fs::{self, DirBuilder, OpenOptions, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};
use std::path::Path;

fn tighten(path: &Path) {
    if fs::metadata(path).is_ok_and(|meta| meta.permissions().mode() & 0o077 != 0) {
        let _ = fs::set_permissions(path, Permissions::from_mode(0o600));
    }
}

/// The text of the file, or nothing when it is absent. A file other users can read is
/// tightened to `0600` first.
pub fn read(path: &Path) -> String {
    tighten(path);
    fs::read_to_string(path).unwrap_or_default()
}

/// Replaces the file atomically. The new file is created `0600` in a `0700` directory, so the
/// token is never readable by another user, not even for an instant.
pub fn write(path: &Path, text: &str) -> io::Result<()> {
    let dir = path.parent().unwrap_or(Path::new("."));
    DirBuilder::new().recursive(true).mode(0o700).create(dir)?;
    let staged = dir.join(format!(".config.yml.new.{}", std::process::id()));
    let _ = fs::remove_file(&staged);
    let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(&staged)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    fs::rename(&staged, path)
}
