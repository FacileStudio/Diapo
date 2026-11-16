//! Reads files the capture worker wrote. The worker runs a browser that loads user HTML, so
//! everything under the captures directory is treated as hostile: a file is opened without
//! following a symbolic link, must be a regular file within a size cap, and must really sit
//! inside the captures directory once opened, whatever links its parents went through.

use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::fd::AsRawFd;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

/// The largest `status.json` or heartbeat the API reads.
pub const SMALL: u64 = 64 * 1024;

/// The largest still or thumbnail the API reads.
pub const PICTURE: u64 = 64 << 20;

/// The largest PDF the API reads.
pub const DOCUMENT: u64 = 300 << 20;

fn refused(why: &str) -> io::Error {
    io::Error::new(io::ErrorKind::PermissionDenied, why.to_owned())
}

fn opened(path: &Path) -> io::Result<File> {
    OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK).open(path)
}

/// Where the open file `file` really is, as the kernel resolved it.
fn resolved(file: &File) -> io::Result<PathBuf> {
    std::fs::read_link(format!("/proc/self/fd/{}", file.as_raw_fd()))
}

/// The bytes of `path`, a regular file of at most `max` bytes inside `root`. Blocking.
pub fn read_blocking(root: &Path, path: &Path, max: u64) -> io::Result<Vec<u8>> {
    let file = opened(path)?;
    let meta = file.metadata()?;
    if !meta.is_file() || meta.len() > max {
        return Err(refused("a capture output must be a regular file within its size cap"));
    }
    if !resolved(&file)?.starts_with(root.canonicalize()?) {
        return Err(refused("a capture output must sit inside the captures directory"));
    }
    let mut bytes = Vec::with_capacity(meta.len() as usize);
    file.take(max + 1).read_to_end(&mut bytes)?;
    match bytes.len() as u64 > max {
        true => Err(refused("a capture output grew past its size cap")),
        false => Ok(bytes),
    }
}

/// [`read_blocking`], off the async threads.
pub async fn read(root: &Path, path: &Path, max: u64) -> io::Result<Vec<u8>> {
    let (root, path) = (root.to_owned(), path.to_owned());
    tokio::task::spawn_blocking(move || read_blocking(&root, &path, max)).await.map_err(io::Error::other)?
}

/// Whether `path` is a real directory inside `root`, not a link to one.
pub fn real_dir(root: &Path, path: &Path) -> bool {
    let plain = path.symlink_metadata().is_ok_and(|meta| meta.is_dir());
    let inside = path.canonicalize().ok().zip(root.canonicalize().ok());
    plain && inside.is_some_and(|(path, root)| path.starts_with(root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_fifos_large_files_and_escapes_are_refused() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let secret = outside.path().join("environ");
        std::fs::write(&secret, "SECRET=1").unwrap();
        let job = root.path().join("j1");
        std::fs::create_dir(&job).unwrap();
        std::fs::write(job.join("out.pdf"), "%PDF").unwrap();
        assert_eq!(read_blocking(root.path(), &job.join("out.pdf"), 10).unwrap(), b"%PDF");
        assert!(read_blocking(root.path(), &job.join("out.pdf"), 3).is_err());
        std::os::unix::fs::symlink(&secret, job.join("linked.pdf")).unwrap();
        assert!(read_blocking(root.path(), &job.join("linked.pdf"), 100).is_err());
        std::os::unix::fs::symlink(outside.path(), job.join("stills")).unwrap();
        assert!(read_blocking(root.path(), &job.join("stills/environ"), 100).is_err());
        assert!(!real_dir(root.path(), &job.join("stills")) && real_dir(root.path(), &job));
        std::os::unix::fs::symlink(outside.path(), root.path().join("j2")).unwrap();
        assert!(read_blocking(root.path(), &root.path().join("j2/environ"), 100).is_err());
        let fifo = job.join("fifo");
        assert!(std::process::Command::new("mkfifo").arg(&fifo).status().unwrap().success());
        assert!(read_blocking(root.path(), &fifo, 100).is_err());
        std::fs::create_dir(job.join("dir")).unwrap();
        assert!(read_blocking(root.path(), &job.join("dir"), 100).is_err());
    }
}
