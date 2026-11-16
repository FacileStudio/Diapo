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

use std::collections::BTreeMap;

use serde_json::Value;

use crate::api::Client;
use crate::config;
use crate::failure::{Failure, Outcome};
use crate::sync::{Hashes, Link, asset_name, safe_name};

/// The assets a server holds for a deck.
struct RemoteAssets {
    /// Content hashes by name under `assets/`.
    hashes: Hashes,
    /// The server's own spelling of each name.
    spelled: BTreeMap<String, String>,
}

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

/// Lists the deck's assets, refusing a name that would leave the `assets/` folder.
fn remote_assets(client: &Client, deck: &str) -> Outcome<RemoteAssets> {
    let mut remote = RemoteAssets { hashes: Hashes::new(), spelled: BTreeMap::new() };
    for asset in client.assets(deck)? {
        let name = asset_name(&asset.name).to_owned();
        if !safe_name(&name) {
            return Err(Failure::new(
                "bad_asset_name",
                format!("the server lists an asset with an unsafe name: {name}"),
            ));
        }
        remote.hashes.insert(name.clone(), asset.sha256);
        remote.spelled.insert(name, asset.name);
    }
    Ok(remote)
}

/// Prints the one JSON document a `--json` run puts on stdout.
fn print_json(value: &Value) {
    println!("{}", serde_json::to_string_pretty(value).unwrap_or_default());
}
