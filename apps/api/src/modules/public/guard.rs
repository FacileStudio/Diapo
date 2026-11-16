use std::sync::Arc;

use tronc::http::{RateLimitConfig, RateLimiter, SecurityOverride, SecurityOverrideConfig, Sources};

/// The limit of the asset route of a shared frame, which a deck with many pictures calls many
/// times a page: 30 requests a second per client, with a burst of 300.
pub fn loose() -> Arc<RateLimiter> {
    Arc::new(RateLimiter::new(RateLimitConfig { limit: 30.0, burst: 300, ..RateLimitConfig::default() }))
}

/// The limit of the public routes: 5 requests a second per client, with a burst of 60.
pub fn limiter() -> Arc<RateLimiter> {
    Arc::new(RateLimiter::new(RateLimitConfig { limit: 5.0, burst: 60, ..RateLimitConfig::default() }))
}

/// The chassis override that lets `ancestors` and the app itself frame a route, and drops
/// `X-Frame-Options` there.
pub fn framable(ancestors: &[String]) -> std::result::Result<Arc<SecurityOverride>, String> {
    let mut sources = vec!["'self'".to_owned()];
    sources.extend(ancestors.iter().cloned());
    let config = SecurityOverrideConfig {
        csp_sources: Sources::from([("frame-ancestors".to_owned(), sources)]),
        ..SecurityOverrideConfig::default()
    };
    SecurityOverride::new(config).map(Arc::new)
}
