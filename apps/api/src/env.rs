use std::path::PathBuf;

use tronc::env::{Core, EnvError, Vars};

use crate::modules::assets::sniff::{ASSET_MAX_BYTES, DECK_ASSETS_MAX_BYTES};
use crate::modules::assistant::Bulle;
use crate::modules::decks::source::SOURCE_MAX_BYTES;

/// The byte caps one deck is held to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// `SOURCE_MAX_BYTES`: the largest deck source.
    pub source: usize,
    /// `ASSET_MAX_BYTES`: the largest single asset.
    pub asset: usize,
    /// `DECK_ASSETS_MAX_BYTES`: the most bytes the assets of one deck hold together.
    pub deck_assets: usize,
}

/// Everything the binary reads from its environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// The chassis variables: port, database, proxies, Journal.
    pub core: Core,
    /// Registre sign-in. `sso_only` is always true.
    pub porte: porte::Config,
    /// `PORTE_TOKEN_KEY`: seals the stored OIDC tokens, and the frame asset links derive their key
    /// from it.
    pub token_key: String,
    /// `DATA_DIR`: where assets live.
    pub data_dir: PathBuf,
    /// `MCP_RESOURCE`: turns `/mcp` on.
    pub mcp_resource: String,
    /// `MCP_STATE_KEY`: the HMAC key of the confirmation flow.
    pub mcp_state_key: String,
    /// The byte caps.
    pub limits: Limits,
    /// `BULLE_URL` and `BULLE_CLIENT_ID`: where the person's assistant lives.
    pub bulle: Bulle,
}

impl Config {
    /// Reads the process environment.
    pub fn load() -> Result<Self, EnvError> {
        Self::from_vars(&Vars::process())
    }

    /// Reads `vars`, refusing a production boot without Registre.
    pub fn from_vars(vars: &Vars) -> Result<Self, EnvError> {
        let core = Core::from_vars(vars, true)?;
        let text = |key: &str| vars.string(key, "").trim().to_owned();
        let porte = porte::Config { app_name: "Diapo".to_owned(), sso_only: true, ..porte::Config::from_vars(vars)? };
        let limits = Limits {
            source: bytes(vars, "SOURCE_MAX_BYTES", SOURCE_MAX_BYTES)?,
            asset: bytes(vars, "ASSET_MAX_BYTES", ASSET_MAX_BYTES)?,
            deck_assets: bytes(vars, "DECK_ASSETS_MAX_BYTES", DECK_ASSETS_MAX_BYTES)?,
        };
        let config = Config {
            porte,
            token_key: text("PORTE_TOKEN_KEY"),
            data_dir: PathBuf::from(vars.string("DATA_DIR", "/data")),
            mcp_resource: text("MCP_RESOURCE"),
            mcp_state_key: text("MCP_STATE_KEY"),
            limits,
            bulle: Bulle::from_vars(vars)?,
            core,
        };
        config.check()?;
        Ok(config)
    }

    fn check(&self) -> Result<(), EnvError> {
        if self.core.is_production() && self.porte.issuer.is_empty() {
            return Err(EnvError("OIDC_ISSUER is required in production".to_owned()));
        }
        if self.core.is_production() && self.token_key.is_empty() {
            return Err(EnvError("PORTE_TOKEN_KEY is required in production".to_owned()));
        }
        if !self.mcp_resource.is_empty() && self.mcp_state_key.is_empty() {
            return Err(EnvError("MCP_STATE_KEY is required with MCP_RESOURCE".to_owned()));
        }
        Ok(())
    }
}

fn bytes(vars: &Vars, key: &str, default: usize) -> Result<usize, EnvError> {
    let raw = vars.int(key, default as i64)?;
    match usize::try_from(raw) {
        Ok(max) if max > 0 => Ok(max),
        _ => Err(EnvError(format!("{key} must be a positive number of bytes, not {raw}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(extra: &[(&str, &str)]) -> Vars {
        let mut pairs = vec![("DATABASE_URL", "postgres://x/y")];
        pairs.extend_from_slice(extra);
        Vars::from_pairs(pairs)
    }

    #[test]
    fn production_needs_the_issuer_and_the_token_key() {
        let error = Config::from_vars(&vars(&[("APP_ENV", "production")])).unwrap_err();
        assert!(error.0.contains("OIDC_ISSUER"), "{error}");
        let pairs = [("APP_ENV", "production"), ("OIDC_ISSUER", "https://sso.test")];
        let error = Config::from_vars(&vars(&pairs)).unwrap_err();
        assert!(error.0.contains("PORTE_TOKEN_KEY") || error.0.contains("OIDC"), "{error}");
    }

    #[test]
    fn mcp_needs_a_state_key() {
        let error = Config::from_vars(&vars(&[("MCP_RESOURCE", "https://diapo.test/mcp")])).unwrap_err();
        assert!(error.0.contains("MCP_STATE_KEY"), "{error}");
    }

    #[test]
    fn defaults_are_sso_only_with_slash_data_positive_caps_and_the_suite_bulle() {
        let config = Config::from_vars(&vars(&[])).unwrap();
        assert!(config.porte.sso_only);
        assert_eq!(config.porte.app_name, "Diapo");
        assert_eq!(config.data_dir, PathBuf::from("/data"));
        assert_eq!(config.limits, Limits { source: 512 << 10, asset: 25 << 20, deck_assets: 200 << 20 });
        assert_eq!(config.bulle.url, "https://bulle.facile.studio");
        let config = Config::from_vars(&vars(&[("SOURCE_MAX_BYTES", "4096"), ("ASSET_MAX_BYTES", "1024")])).unwrap();
        assert_eq!((config.limits.source, config.limits.asset), (4096, 1024));
        for bad in [("SOURCE_MAX_BYTES", "0"), ("ASSET_MAX_BYTES", "-1"), ("DECK_ASSETS_MAX_BYTES", "lots")] {
            assert!(Config::from_vars(&vars(&[bad])).is_err(), "{bad:?}");
        }
    }
}
