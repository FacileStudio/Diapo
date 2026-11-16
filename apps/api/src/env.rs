use tronc::env::{Core, EnvError, Vars};

/// Everything the binary reads from its environment.
#[derive(Debug, Clone)]
pub struct Config {
    /// The chassis variables: port, database, proxies, Journal.
    pub core: Core,
    /// Registre sign-in. `sso_only` is always true.
    pub porte: porte::Config,
    /// `PORTE_TOKEN_KEY`: seals the stored OIDC tokens.
    pub token_key: String,
}

impl Config {
    /// Reads the process environment.
    pub fn load() -> Result<Self, EnvError> {
        Self::from_vars(&Vars::process())
    }

    /// Reads `vars`, refusing a production boot without Registre.
    pub fn from_vars(vars: &Vars) -> Result<Self, EnvError> {
        let core = Core::from_vars(vars, true)?;
        let porte = porte::Config { app_name: "Diapo".to_owned(), sso_only: true, ..porte::Config::from_vars(vars)? };
        let config = Config { porte, token_key: vars.string("PORTE_TOKEN_KEY", "").trim().to_owned(), core };
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
        Ok(())
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
    fn production_needs_the_issuer() {
        let error = Config::from_vars(&vars(&[("APP_ENV", "production")])).unwrap_err();
        assert!(error.0.contains("OIDC_ISSUER"), "{error}");
    }

    #[test]
    fn production_needs_the_token_key() {
        let pairs = [("APP_ENV", "production"), ("OIDC_ISSUER", "https://sso.test")];
        let error = Config::from_vars(&vars(&pairs)).unwrap_err();
        assert!(error.0.contains("PORTE_TOKEN_KEY") || error.0.contains("OIDC"), "{error}");
    }

    #[test]
    fn development_is_sso_only_without_registre() {
        let config = Config::from_vars(&vars(&[])).unwrap();
        assert!(config.porte.sso_only);
        assert_eq!(config.porte.app_name, "Diapo");
        assert!(config.token_key.is_empty());
    }
}
