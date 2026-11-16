//! Watching the deck folder, and reloading the open pages once a burst of changes settles.

use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};

use super::{Site, page};
use crate::failure::{Failure, Outcome};
use crate::ui;

const QUIET: Duration = Duration::from_millis(80);

fn hidden(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        let name = name.to_string_lossy();
        name.starts_with('.') || name.ends_with('~')
    })
}

fn relevant(event: &Event) -> bool {
    let changed = event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove();
    changed && event.paths.iter().any(|path| !hidden(path))
}

fn settle(changes: &Receiver<()>, site: &Site) {
    while changes.recv().is_ok() {
        while changes.recv_timeout(QUIET).is_ok() {}
        ui::step(&format!("Reloading {}", site.folder.label()));
        page::report(site);
        site.hub.reload();
    }
}

/// Watches the deck folder. The watch stops when the answer is dropped.
pub fn start(site: &Arc<Site>) -> Outcome<RecommendedWatcher> {
    let fail =
        |error: notify::Error| Failure::new("watch", format!("cannot watch {}: {error}", site.folder.dir.display()));
    let (sender, changes) = mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event: notify::Result<Event>| {
        if event.is_ok_and(|event| relevant(&event)) {
            let _ = sender.send(());
        }
    })
    .map_err(fail)?;
    watcher.watch(&site.folder.dir, RecursiveMode::Recursive).map_err(fail)?;
    let site = Arc::clone(site);
    thread::spawn(move || settle(&changes, &site));
    Ok(watcher)
}
