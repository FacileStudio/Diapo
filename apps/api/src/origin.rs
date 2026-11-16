//! The origins the app is reached at and may be framed from.

use tronc::env::{Core, EnvError, Vars};

/// The origin Prisme frames embeds from, allowed when `EMBED_ANCESTORS` is unset.
pub const DEFAULT_ANCESTOR: &str = "https://prisme.facile.studio";

fn origin(key: &str, raw: &str) -> Result<String, EnvError> {
    let refused = || EnvError(format!("{key} {raw:?} must be an http(s) origin, without a path"));
    let url = url::Url::parse(raw.trim()).map_err(|_| refused())?;
    let bare = matches!(url.scheme(), "https" | "http") && url.host_str().is_some_and(|host| !host.is_empty());
    bare.then(|| url.origin().ascii_serialization()).ok_or_else(refused)
}

/// `PUBLIC_URL`, else the origin of `OIDC_SUCCESS_URL`, else `http://localhost:<PORT>`.
pub fn public_url(vars: &Vars, core: &Core, porte: &porte::Config) -> Result<String, EnvError> {
    let given = vars.string("PUBLIC_URL", "");
    match (given.trim().is_empty(), porte.success_url.trim().is_empty()) {
        (false, _) => origin("PUBLIC_URL", &given),
        (true, false) => origin("OIDC_SUCCESS_URL", &porte.success_url),
        (true, true) => Ok(format!("http://localhost:{}", core.port)),
    }
}

/// `EMBED_ANCESTORS` as origins, [`DEFAULT_ANCESTOR`] when unset.
pub fn ancestors(vars: &Vars) -> Result<Vec<String>, EnvError> {
    let raw = vars.string("EMBED_ANCESTORS", DEFAULT_ANCESTOR);
    raw.split([' ', ',']).filter(|part| !part.is_empty()).map(|part| origin("EMBED_ANCESTORS", part)).collect()
}
