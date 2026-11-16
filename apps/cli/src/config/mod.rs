//! Where the CLI finds its server and its token: a flag, then the environment, then
//! `${XDG_CONFIG_HOME:-~/.config}/diapo/config.yml`, then the built-in default.

mod file;
pub mod flat;

use std::env;
use std::io;
use std::path::PathBuf;

/// The hosted instance, used when nothing else names a server.
pub const DEFAULT_SERVER: &str = "https://diapo.facile.studio";

/// The environment variable that overrides the stored token.
pub const ENV_TOKEN: &str = "DIAPO_TOKEN";

/// The environment variable that overrides the stored server.
pub const ENV_SERVER: &str = "DIAPO_SERVER_URL";

const TOKEN: &str = "token";
const SERVER: &str = "server_url";

/// The server to call and the token to call it with.
#[derive(Debug, Clone, PartialEq)]
pub struct Settings {
    /// The site root, without `/api`.
    pub server: String,
    /// The bearer token, when there is one for this server.
    pub token: Option<String>,
}

/// The config file: `${XDG_CONFIG_HOME:-~/.config}/diapo/config.yml` on every platform.
pub fn path() -> PathBuf {
    let base = variable("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env::var_os("HOME").unwrap_or_default()).join(".config"));
    base.join("diapo").join("config.yml")
}

/// The site root of what a person typed or a tool stored: no trailing slash, no `/api`.
pub fn site(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    trimmed.strip_suffix("/api").unwrap_or(trimmed).to_owned()
}

fn variable(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.trim().is_empty())
}

/// What the config file holds: its server, or the default, and its token.
pub fn stored() -> Settings {
    let text = file::read(&path());
    Settings {
        server: flat::get(&text, SERVER).map_or_else(|| DEFAULT_SERVER.to_owned(), |url| site(&url)),
        token: flat::get(&text, TOKEN),
    }
}

/// Resolves the settings: flag, then environment, then file, then default. The stored token
/// is used only for the server it was issued by.
pub fn resolve(flag: Option<&str>) -> Settings {
    let file = stored();
    let server = flag.map(site).or_else(|| variable(ENV_SERVER).map(|url| site(&url))).unwrap_or(file.server.clone());
    let token = variable(ENV_TOKEN).or(file.token.filter(|_| file.server == server));
    Settings { server, token }
}

/// Stores a login, keeping every other line of the file.
pub fn store_login(server: &str, token: &str) -> io::Result<()> {
    let text = file::read(&path());
    file::write(&path(), &flat::set(&flat::set(&text, SERVER, server), TOKEN, token))
}

/// Removes the stored token, keeping every other line. Answers whether there was one.
pub fn clear_token() -> io::Result<bool> {
    let text = file::read(&path());
    if flat::get(&text, TOKEN).is_none() {
        return Ok(false);
    }
    file::write(&path(), &flat::remove(&text, TOKEN)).map(|()| true)
}
