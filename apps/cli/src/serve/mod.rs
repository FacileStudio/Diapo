//! `diapo serve`: the deck on localhost, recompiled on every request and reloaded in the
//! browser on every change to its folder.

mod events;
mod files;
mod page;
mod watch;

use std::sync::{Arc, Mutex};

use serde_json::json;
use tiny_http::{Request, Server};

use crate::cli::ServeArgs;
use crate::failure::{Failure, Outcome};
use crate::project::Folder;
use crate::ui;

/// What every request and the watcher share.
pub struct Site {
    /// The deck being served.
    folder: Folder,
    /// The browsers waiting for a reload.
    hub: events::Hub,
    /// The last document that compiled, shown under the error overlay.
    last_good: Mutex<Option<String>>,
}

fn route(site: &Site, request: Request) {
    let path = request.url().split(['?', '#']).next().unwrap_or("/").to_owned();
    match path.as_str() {
        "/" | "/index.html" => {
            files::respond(request, 200, "text/html; charset=utf-8", page::render(site).into_bytes())
        }
        page::EVENTS => site.hub.subscribe(request),
        _ => files::serve(&site.folder.dir, &path, request),
    }
}

/// Serves until the process is interrupted.
pub fn run(args: &ServeArgs) -> Outcome {
    let folder = Folder::locate(args.dir.as_deref())?;
    let server = Server::http(("127.0.0.1", args.port)).map_err(|error| {
        Failure::new(
            "port",
            format!("cannot listen on port {} ({error}) — pass `-p <port>` to pick another", args.port),
        )
    })?;
    let port = server.server_addr().to_ip().map_or(args.port, |address| address.port());
    let site = Arc::new(Site { folder, hub: events::Hub::default(), last_good: Mutex::new(None) });
    let _watcher = watch::start(&site)?;
    let url = format!("http://localhost:{port}");
    page::report(&site);
    if ui::json() {
        println!("{}", json!({ "url": url, "file": site.folder.label() }));
    }
    ui::success(&format!("Serving {} at {url}", site.folder.label()));
    ui::hint("Arrow keys move through the deck, Ctrl+C stops the server");
    server.incoming_requests().for_each(|request| route(&site, request));
    Ok(())
}
