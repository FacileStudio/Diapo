use std::path::{Path, PathBuf};
use std::time::Duration;

use capture::job::{HEARTBEAT, Kind, Layout, Spec, State, Status, now};

use super::App;
use super::upload::PNG;

pub const JPEG: &[u8] = b"\xff\xd8\xff-a-thumbnail";

fn write(layout: &Layout, spec: &Spec) {
    match spec.kind {
        Kind::Thumbnail => std::fs::write(layout.output(Kind::Thumbnail), JPEG).unwrap(),
        Kind::Pdf => {
            let facts =
                format!("%PDF-1.7 steps={} bundle={}", spec.steps, layout.bundle().join("index.html").is_file());
            std::fs::write(layout.output(Kind::Pdf), facts).unwrap()
        }
        Kind::Stills => {
            std::fs::create_dir_all(layout.output(Kind::Stills)).unwrap();
            for scene in spec.scenes.clone().unwrap_or_default() {
                std::fs::write(layout.still(scene), PNG).unwrap();
            }
        }
    }
}

fn serve(captures: &Path) {
    std::fs::create_dir_all(captures).unwrap();
    std::fs::write(captures.join(HEARTBEAT), now()).unwrap();
    let jobs: Vec<PathBuf> = std::fs::read_dir(captures).unwrap().flatten().map(|entry| entry.path()).collect();
    for dir in jobs.into_iter().filter(|dir| dir.join("job.json").is_file() && !dir.join("status.json").exists()) {
        let layout = Layout { dir };
        let spec: Spec = serde_json::from_slice(&std::fs::read(layout.spec()).unwrap()).unwrap();
        write(&layout, &spec);
        let status = Status { state: State::Done, done: 1, total: 1, error: None, updated_at: now() };
        std::fs::write(layout.status(), serde_json::to_vec(&status).unwrap()).unwrap();
    }
}

pub fn start(app: &App) -> tokio::task::JoinHandle<()> {
    let captures = app.state.captures.dir().to_owned();
    serve(&captures);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(40)).await;
            serve(&captures);
        }
    })
}

pub async fn until<F, Fut>(what: &str, mut check: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    for _ in 0..200 {
        if check().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("timed out waiting for {what}");
}

fn plant(layout: &Layout, spec: &Spec, secret: &Path) {
    let link = |target: &Path, at: PathBuf| std::os::unix::fs::symlink(target, at).unwrap();
    match spec.kind {
        Kind::Thumbnail => link(secret, layout.output(Kind::Thumbnail)),
        Kind::Pdf => link(secret, layout.output(Kind::Pdf)),
        Kind::Stills => link(secret.parent().unwrap(), layout.output(Kind::Stills)),
    }
}

fn hostile(captures: &Path, secret: &Path) {
    std::fs::create_dir_all(captures).unwrap();
    std::fs::write(captures.join(HEARTBEAT), now()).unwrap();
    let jobs: Vec<PathBuf> = std::fs::read_dir(captures).unwrap().flatten().map(|entry| entry.path()).collect();
    for dir in jobs.into_iter().filter(|dir| dir.join("job.json").is_file() && !dir.join("status.json").exists()) {
        let layout = Layout { dir };
        let spec: Spec = serde_json::from_slice(&std::fs::read(layout.spec()).unwrap()).unwrap();
        plant(&layout, &spec, secret);
        let status = Status { state: State::Done, done: 1, total: 1, error: None, updated_at: now() };
        std::fs::write(layout.status(), serde_json::to_vec(&status).unwrap()).unwrap();
    }
}

/// A worker that answers every job `done` with its outputs replaced by symbolic links to
/// `secret`, a file outside the captures directory, as a compromised browser could.
pub fn start_hostile(app: &App, secret: PathBuf) -> tokio::task::JoinHandle<()> {
    let captures = app.state.captures.dir().to_owned();
    hostile(&captures, &secret);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(40)).await;
            hostile(&captures, &secret);
        }
    })
}
