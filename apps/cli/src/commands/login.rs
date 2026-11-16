//! `diapo login`: asks the server how it signs people in, runs the flow, stores the token.

use std::env;

use serde_json::json;

use crate::api::Client;
use crate::api::device::ISSUER;
use crate::cli::{Globals, LoginArgs};
use crate::commands::print_json;
use crate::config::{self, ENV_TOKEN};
use crate::failure::{Failure, Outcome};
use crate::login::{device, loopback};
use crate::{browser, ui};

/// The environment variable that names another identity provider, for a self-hosted suite.
pub const ENV_ISSUER: &str = "DIAPO_ISSUER";

fn by_device(args: &LoginArgs, client: &Client) -> Outcome<bool> {
    if !args.device && browser::has_display() {
        return Ok(false);
    }
    let mounted = client.device_mounted()?;
    if args.device && !mounted {
        let message = "this server has no device sign-in — run `diapo login` without --device";
        return Err(Failure::new("no_device_login", message));
    }
    Ok(mounted)
}

fn token(args: &LoginArgs, client: &Client) -> Outcome<String> {
    if !client.auth_config()?.oidc_enabled {
        let message = format!("{} has no single sign-on — ask its administrator to configure one", client.server());
        return Err(Failure::new("no_sso", message));
    }
    if by_device(args, client)? {
        let issuer = env::var(ENV_ISSUER).ok().filter(|issuer| !issuer.is_empty());
        return device::sign_in(client, issuer.as_deref().unwrap_or(ISSUER), !args.no_browser);
    }
    loopback::sign_in(client, !args.no_browser)
}

/// Signs in to the server named on the command line, or the stored one, or the default.
pub fn run(args: &LoginArgs, globals: &Globals) -> Outcome {
    let server = config::resolve(args.target.as_deref().or(globals.server.as_deref())).server;
    let token = token(args, &Client::new(&server, None))?;
    config::store_login(&server, &token)?;
    let email = Client::new(&server, Some(&token)).me().map(|me| me.email).unwrap_or_default();
    if env::var_os(ENV_TOKEN).is_some() {
        ui::warn(&format!("{ENV_TOKEN} is set and takes precedence over this login"));
    }
    if ui::json() {
        print_json(&json!({ "server": server, "email": email }));
        return Ok(());
    }
    let who = if email.is_empty() { String::new() } else { format!(" as {email}") };
    ui::success(&format!("Signed in to {server}{who}"));
    ui::hint(&format!("The token is stored in {}", config::path().display()));
    Ok(())
}
