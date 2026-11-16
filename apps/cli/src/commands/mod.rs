//! One module per command, and what the server commands share.

pub mod build;
pub mod check;
pub mod export;
pub mod guide;
pub mod list;
pub mod login;
pub mod logout;
pub mod new;
pub mod open;
pub mod pull;
pub mod push;
pub mod whoami;

use serde_json::Value;

use crate::api::Client;
use crate::config;
use crate::failure::{Failure, Outcome};
use crate::sync::{Hashes, Link, safe_name};

/// A client for the resolved server, or the reason there is no login to call it with.
fn session(server: Option<&str>) -> Outcome<Client> {
    let settings = config::resolve(server);
    match settings.token {
        Some(token) => Ok(Client::new(&settings.server, Some(&token))),
        None => {
            let server = settings.server;
            Err(Failure::new("not_logged_in", format!("not logged in to {server} — run `diapo login {server}`")))
        }
    }
}

/// A client for the server a linked folder belongs to. `--server` may not name another one.
fn linked_session(flag: Option<&str>, link: &Link) -> Outcome<Client> {
    if flag.is_some_and(|flag| config::site(flag) != link.server) {
        let message = format!("this folder is linked to a deck on {} — drop --server", link.server);
        return Err(Failure::new("other_server", message));
    }
    session(Some(&link.server))
}

/// Lists the deck's assets by name, refusing a name that would leave the deck folder.
fn remote_assets(client: &Client, deck: &str) -> Outcome<Hashes> {
    let mut remote = Hashes::new();
    for asset in client.assets(deck)? {
        if !safe_name(&asset.name) {
            let message = format!("the server lists an asset with an unsafe name: {}", asset.name);
            return Err(Failure::new("bad_asset_name", message));
        }
        remote.insert(asset.name, asset.sha256);
    }
    Ok(remote)
}

/// Prints the one JSON document a `--json` run puts on stdout.
fn print_json(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
}
