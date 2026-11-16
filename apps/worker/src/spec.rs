//! `job.json` and `status.json` on disk, checked against the worker's own limits.

use std::path::Path;

use capture::job::{Layout, Spec, State, Status, named, now, parse};

use crate::config::Limits;
use crate::files;

/// A status written now.
pub(crate) fn status(state: State, done: u32, total: u32) -> Status {
    Status { state, done, total, error: None, updated_at: now() }
}

/// A failure written now.
pub(crate) fn failed(reason: impl Into<String>) -> Status {
    Status { error: Some(reason.into()), ..status(State::Failed, 0, 0) }
}

/// Writes the status of the job in `dir`, atomically.
pub(crate) fn save(dir: &Path, status: &Status) {
    let text = serde_json::to_string(status).unwrap_or_default();
    let _ = files::write(&Layout { dir: dir.to_owned() }.status(), text.as_bytes());
}

/// The status of the job in `dir`: `None` without a file, an error for a file that is not one.
pub(crate) fn load_status(dir: &Path) -> Option<Result<Status, String>> {
    let path = Layout { dir: dir.to_owned() }.status();
    let parsed = |text: String| serde_json::from_str(&text).map_err(|error| error.to_string());
    files::exists(&path).then(|| files::read(&path).and_then(parsed))
}

fn check(spec: &Spec, limits: &Limits) -> Result<(), String> {
    let side = |pixels: u32| (16..=limits.side).contains(&pixels);
    if !side(spec.width) || !side(spec.height) {
        return Err(format!("{}x{} must be 16 to {} pixels a side", spec.width, spec.height, limits.side));
    }
    if spec.scenes.as_ref().is_some_and(|scenes| scenes.len() as u64 > u64::from(limits.pages)) {
        return Err(format!("a job shoots at most {} scenes", limits.pages));
    }
    match parse(&spec.created_at) {
        Some(_) => Ok(()),
        None => Err("created_at must be an RFC 3339 timestamp".to_owned()),
    }
}

/// Reads and checks the `job.json` of the job directory `dir`.
pub(crate) fn load(dir: &Path, limits: &Limits) -> Result<Spec, String> {
    let text = files::read(&Layout { dir: dir.to_owned() }.spec())?;
    let spec: Spec = serde_json::from_str(&text).map_err(|error| format!("job.json: {error}"))?;
    let name = dir.file_name().unwrap_or_default().to_string_lossy();
    if spec.id != name || !named(&spec.id) {
        return Err(format!("job.json names job `{}` in directory `{name}`", spec.id));
    }
    check(&spec, limits).map(|()| spec)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(dir: &Path, json: &str) -> Result<Spec, String> {
        let job = dir.join("j1");
        let _ = std::fs::create_dir(&job);
        std::fs::write(job.join("job.json"), json).unwrap();
        load(&job, &Limits { pages: 2, ..Limits::default() })
    }

    #[test]
    fn a_job_is_held_to_the_limits() {
        let dir = tempfile::tempdir().unwrap();
        let with = |fields: &str| format!(r#"{{"id":"j1","created_at":"2026-11-16T10:00:00Z",{fields}}}"#);
        let good = job(dir.path(), &with(r#""kind":"pdf","width":1920,"height":1080,"steps":true"#)).unwrap();
        assert!(good.steps && good.scenes.is_none());
        for (fields, reason) in [
            (r#""kind":"pdf","width":8000,"height":1080"#, "16 to 4096"),
            (r#""kind":"stills","width":1920,"height":1080,"scenes":[0,1,2]"#, "at most 2 scenes"),
            (r#""kind":"video","width":1920,"height":1080"#, "job.json"),
        ] {
            let error = job(dir.path(), &with(fields)).unwrap_err();
            assert!(error.contains(reason), "{error}");
        }
        let other = r#"{"id":"j2","kind":"pdf","width":16,"height":16,"created_at":"2026-11-16T10:00:00Z"}"#;
        assert!(job(dir.path(), other).unwrap_err().contains("names job `j2`"));
        let undated = r#"{"id":"j1","kind":"pdf","width":16,"height":16,"created_at":"yesterday"}"#;
        assert!(job(dir.path(), undated).unwrap_err().contains("RFC 3339"));
    }
}
