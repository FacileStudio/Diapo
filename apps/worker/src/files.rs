//! File access that stays inside the captures directory: nothing here follows a symbolic link.

use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// The largest `job.json`, `status.json` or heartbeat the worker reads.
const MAX_BYTES: u64 = 64 * 1024;

/// Whether `path` is a real directory, not a link to one.
pub(crate) fn is_dir(path: &Path) -> bool {
    path.symlink_metadata().is_ok_and(|meta| meta.file_type().is_dir())
}

/// Whether anything, a link included, sits at `path`.
pub(crate) fn exists(path: &Path) -> bool {
    path.symlink_metadata().is_ok()
}

/// The text of a small regular file. A link, a directory and a large file are refused.
pub(crate) fn read(path: &Path) -> Result<String, String> {
    let name = path.file_name().unwrap_or_default().to_string_lossy().into_owned();
    let meta = path.symlink_metadata().map_err(|error| format!("{name}: {error}"))?;
    if !meta.file_type().is_file() || meta.len() > MAX_BYTES {
        return Err(format!("{name} must be a regular file of at most {MAX_BYTES} bytes"));
    }
    std::fs::read_to_string(path).map_err(|error| format!("{name}: {error}"))
}

/// Removes whatever sits at `path`. A link is removed, not followed.
pub(crate) fn clear(path: &Path) {
    let _ = if is_dir(path) { std::fs::remove_dir_all(path) } else { std::fs::remove_file(path) };
}

/// The temporary name a file or a directory is built under before it is renamed to `path`.
pub(crate) fn partial(path: &Path) -> std::path::PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".tmp");
    path.with_file_name(name)
}

/// Writes `bytes` to `path` so that a reader sees the old file or the new one, never a part: a
/// fresh temporary file beside it, then a rename.
pub(crate) fn write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let temporary = partial(path);
    clear(&temporary);
    let mut file = OpenOptions::new().write(true).create_new(true).open(&temporary)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std::fs::rename(&temporary, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_are_neither_read_nor_written_through() {
        let dir = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let secret = outside.path().join("secret.json");
        std::fs::write(&secret, "{}").unwrap();
        std::os::unix::fs::symlink(&secret, dir.path().join("job.json")).unwrap();
        std::os::unix::fs::symlink(outside.path(), dir.path().join("bundle")).unwrap();
        assert!(read(&dir.path().join("job.json")).is_err());
        assert!(!is_dir(&dir.path().join("bundle")) && exists(&dir.path().join("bundle")));
        std::os::unix::fs::symlink(&secret, dir.path().join("status.json.tmp")).unwrap();
        std::os::unix::fs::symlink(&secret, dir.path().join("status.json")).unwrap();
        write(&dir.path().join("status.json"), b"new").unwrap();
        assert_eq!(std::fs::read_to_string(&secret).unwrap(), "{}");
        assert_eq!(read(&dir.path().join("status.json")).unwrap(), "new");
        clear(&dir.path().join("bundle"));
        assert!(outside.path().exists() && !exists(&dir.path().join("bundle")));
    }
}
