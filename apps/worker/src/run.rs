//! One job, start to final status.

use std::path::Path;
use std::sync::atomic::Ordering;

use capture::job::{Kind, Layout, Spec, State, Status};
use capture::shoot::{self, Deck};
use capture::{Error, Tools};

use crate::config::{Config, Limits};
use crate::shared::{Shared, THUMBNAIL_WIDTH};
use crate::{files, spec};

fn shoot(layout: &Layout, job: &Spec, tools: &Tools, shared: &Shared, most: u32) -> Result<u32, Error> {
    let bundle = layout.bundle();
    let deck = Deck { bundle: &bundle, width: job.width, height: job.height };
    let output = layout.output(job.kind);
    let partial = files::partial(&output);
    files::clear(&partial);
    let mut progress = |done: u32, total: u32| shared.progress(done, total, most);
    let report = match job.kind {
        Kind::Thumbnail => shoot::thumbnail(&deck, THUMBNAIL_WIDTH.min(job.width), &partial, tools),
        Kind::Pdf => shoot::pdf(&deck, job.steps, &partial, tools, &mut progress),
        Kind::Stills => shoot::stills(&deck, job.scenes.as_deref(), &partial, tools, &mut progress),
    };
    match report {
        Ok(report) => std::fs::rename(&partial, &output).map(|()| report.made).map_err(Error::Io),
        Err(error) => {
            files::clear(&partial);
            Err(error)
        }
    }
}

fn capture(layout: &Layout, job: &Spec, tools: &Tools, limits: &Limits) -> Status {
    let shared = Shared::default();
    let captured = std::thread::scope(|scope| {
        scope.spawn(|| shared.beat(layout, limits.wall));
        let captured = shoot(layout, job, tools, &shared, limits.pages);
        shared.over.store(true, Ordering::Relaxed);
        captured
    });
    shared.verdict(captured, limits)
}

/// Runs the job in `dir` and writes its final status. Nothing outside `dir` is written but the
/// heartbeat.
pub(crate) fn run(dir: &Path, tools: &Tools, config: &Config) -> Status {
    let layout = Layout { dir: dir.to_owned() };
    let status = match spec::load(dir, &config.limits) {
        Err(reason) => spec::failed(reason),
        Ok(_) if files::exists(&layout.cancel()) => spec::status(State::Cancelled, 0, 0),
        Ok(_) if !files::is_dir(&layout.bundle()) => spec::failed("the job has no bundle directory"),
        Ok(job) => {
            files::clear(&layout.output(job.kind));
            spec::save(dir, &spec::status(State::Running, 0, 0));
            capture(&layout, &job, tools, &config.limits)
        }
    };
    spec::save(dir, &status);
    status
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use capture::job::HEARTBEAT;

    use super::*;

    const SKIP: &str = "\n================ SKIPPED: the worker's capture test needs chrome-headless-shell (DIAPO_CHROME, \
        PATH or Playwright's cache) ================\n";

    fn job(captures: &Path, id: &str, fields: &str) -> std::path::PathBuf {
        let dir = captures.join(id);
        std::fs::create_dir_all(dir.join("bundle")).unwrap();
        let project = scene::parse("# One\n\n---\nsteps: 1\n---\n\n## Two\n\n- a {step=1}\n").unwrap();
        std::fs::write(dir.join("bundle/index.html"), scene::compile(&project, &scene::Options::new())).unwrap();
        let json = format!(r#"{{"id":"{id}","width":640,"height":360,"created_at":"2026-11-16T10:00:00Z",{fields}}}"#);
        std::fs::write(dir.join("job.json"), json).unwrap();
        dir
    }

    fn config(captures: &Path, limits: Limits) -> Config {
        Config { captures: captures.to_owned(), poll: Duration::ZERO, limits }
    }

    #[test]
    fn a_job_ends_with_its_output_or_a_reason() {
        let Ok(tools) = Tools::locate() else { return eprintln!("{SKIP}") };
        let captures = tempfile::tempdir().unwrap();
        let normal = config(captures.path(), Limits::default());
        let pdf = job(captures.path(), "pdf", r#""kind":"pdf","steps":true"#);
        let done = run(&pdf, &tools, &normal);
        assert_eq!((done.state, done.done, done.total, done.error.clone()), (State::Done, 3, 3, None));
        assert_eq!(spec::load_status(&pdf).unwrap().unwrap(), done);
        assert!(std::fs::read(pdf.join("out.pdf")).unwrap().starts_with(b"%PDF") && !pdf.join("out.pdf.tmp").exists());
        assert!(files::read(&captures.path().join(HEARTBEAT)).is_ok());
        let stills = job(captures.path(), "stills", r#""kind":"stills","scenes":[1]"#);
        assert_eq!(run(&stills, &tools, &normal).state, State::Done);
        assert!(stills.join("stills/1.png").is_file() && !stills.join("stills.tmp").exists());
        let thumb = job(captures.path(), "thumb", r#""kind":"thumbnail""#);
        assert_eq!(run(&thumb, &tools, &normal).state, State::Done);
        assert!(std::fs::read(thumb.join("thumbnail.jpg")).unwrap().starts_with(b"\xff\xd8"));
        let stopped = job(captures.path(), "stopped", r#""kind":"pdf""#);
        std::fs::write(stopped.join("cancel"), "").unwrap();
        assert_eq!(run(&stopped, &tools, &normal).state, State::Cancelled);
        let many = run(
            &job(captures.path(), "many", r#""kind":"pdf","steps":true"#),
            &tools,
            &config(captures.path(), Limits { pages: 2, ..Limits::default() }),
        );
        assert!(many.error.as_deref().is_some_and(|error| error.contains("more than 2 pages")), "{many:?}");
        let empty = job(captures.path(), "empty", r#""kind":"pdf""#);
        std::fs::remove_dir_all(empty.join("bundle")).unwrap();
        assert_eq!(run(&empty, &tools, &normal).error.as_deref(), Some("the job has no bundle directory"));
        assert!(!stopped.join("out.pdf").exists() && !captures.path().join("many/out.pdf").exists());
    }
}
