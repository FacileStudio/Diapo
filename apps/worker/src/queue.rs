//! The queue is the captures directory: one directory per job, queued while it has a `job.json`
//! and no `status.json`.

use std::path::{Path, PathBuf};

use capture::job::{State, named, parse};

use crate::config::Config;
use crate::{files, spec};

/// What a job that was `running` when the worker started is told.
pub(crate) const STOPPED: &str = "the capture worker stopped during this job; start it again";

fn job_dirs(captures: &Path) -> Vec<PathBuf> {
    let entries = std::fs::read_dir(captures).into_iter().flatten().flatten();
    let plain = entries.filter(|entry| entry.file_name().to_str().is_some_and(named));
    plain
        .map(|entry| entry.path())
        .filter(|path| files::is_dir(path) && files::exists(&path.join("job.json")))
        .collect()
}

/// Marks every job left `running`, or with a status that cannot be read, as failed. The worker
/// runs one job at a time, so between two jobs a `running` status is always a leftover.
pub(crate) fn recover(config: &Config) -> usize {
    let stale = |dir: &PathBuf| match spec::load_status(dir) {
        Some(Ok(status)) => status.state == State::Running,
        Some(Err(_)) => true,
        None => false,
    };
    let left: Vec<PathBuf> = job_dirs(&config.captures).into_iter().filter(stale).collect();
    left.iter().for_each(|dir| spec::save(dir, &spec::failed(STOPPED)));
    left.len()
}

/// The queued job to run now: the one queued first. A job whose `job.json` cannot be read sorts
/// first, so it fails at once instead of waiting.
pub(crate) fn next(config: &Config) -> Option<PathBuf> {
    let queued = job_dirs(&config.captures).into_iter().filter(|dir| spec::load_status(dir).is_none());
    let dated = queued.map(|dir| {
        let created = spec::load(&dir, &config.limits).ok().and_then(|spec| parse(&spec.created_at));
        (created.map_or(0, |at| at.unix_timestamp()), dir)
    });
    dated.min().map(|(_, dir)| dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Limits;

    fn job(captures: &Path, id: &str, created: &str) -> PathBuf {
        let dir = captures.join(id);
        std::fs::create_dir_all(dir.join("bundle")).unwrap();
        let json = format!(r#"{{"id":"{id}","kind":"pdf","width":320,"height":180,"created_at":"{created}"}}"#);
        std::fs::write(dir.join("job.json"), json).unwrap();
        dir
    }

    #[test]
    fn the_oldest_queued_job_is_next_and_a_job_left_running_fails() {
        let captures = tempfile::tempdir().unwrap();
        let config =
            Config { captures: captures.path().to_owned(), poll: Default::default(), limits: Limits::default() };
        assert_eq!(next(&config), None);
        let late = job(captures.path(), "late", "2026-11-16T10:00:05Z");
        let early = job(captures.path(), "early", "2026-11-16T12:00:00+02:00");
        let crashed = job(captures.path(), "crashed", "2026-11-16T09:00:00Z");
        let done = job(captures.path(), "done", "2026-11-16T08:00:00Z");
        std::fs::create_dir(captures.path().join(".hidden")).unwrap();
        std::os::unix::fs::symlink(&late, captures.path().join("linked")).unwrap();
        spec::save(&crashed, &spec::status(State::Running, 1, 3));
        spec::save(&done, &spec::status(State::Done, 3, 3));
        assert_eq!(recover(&config), 1);
        let status = spec::load_status(&crashed).unwrap().unwrap();
        assert_eq!((status.state, status.error.as_deref()), (State::Failed, Some(STOPPED)));
        assert_eq!(spec::load_status(&done).unwrap().unwrap().state, State::Done);
        assert_eq!(next(&config), Some(early.clone()));
        spec::save(&early, &spec::status(State::Done, 1, 1));
        assert_eq!(next(&config), Some(late));
    }
}
