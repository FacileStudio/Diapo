//! The API reference at `/docs`: one row per route, checked against the router by a test.

use utoipa::openapi::path::{HttpMethod, OperationBuilder, PathItem};
use utoipa::openapi::response::ResponseBuilder;
use utoipa::openapi::{InfoBuilder, OpenApi, OpenApiBuilder, PathsBuilder};

/// One documented route: method, path under `/api` (or the root), tag, summary, success status.
type Row = (&'static str, &'static str, &'static str, &'static str, &'static str);

const ROUTES: &[Row] = &[
    ("GET", "/version", "meta", "The release this binary was built from", "200"),
    ("GET", "/auth/config", "auth", "Whether sign-in is SSO only and OIDC is on", "200"),
    ("GET", "/auth/oidc", "auth", "Starts Registre sign-in (302)", "302"),
    ("GET", "/auth/oidc/callback", "auth", "Registre's redirect back, sets the session cookie (302)", "302"),
    ("POST", "/auth/oidc/exchange", "auth", "Trades a CLI one-time code for a session token", "200"),
    ("POST", "/auth/oidc/device/exchange", "auth", "Trades a facile-cli device token for a session token", "200"),
    ("POST", "/auth/sync-profile", "auth", "Refreshes the profile from Registre", "200"),
    ("POST", "/auth/backchannel-logout", "auth", "Registre's back-channel logout", "200"),
    ("POST", "/auth/logout", "auth", "Ends the session", "200"),
    ("GET", "/auth/me", "auth", "The signed-in person, or 401", "200"),
    ("GET", "/auth/session", "auth", "The signed-in person, or {\"user\": null}", "200"),
];

fn method(name: &str) -> HttpMethod {
    match name {
        "POST" => HttpMethod::Post,
        _ => HttpMethod::Get,
    }
}

/// The OpenAPI document of every route in [`ROUTES`].
pub fn document() -> OpenApi {
    let paths = ROUTES.iter().fold(PathsBuilder::new(), |paths, (verb, path, tag, summary, status)| {
        let operation = OperationBuilder::new()
            .tag(*tag)
            .summary(Some(*summary))
            .response(*status, ResponseBuilder::new().description(*summary).build());
        paths.path(*path, PathItem::new(method(verb), operation))
    });
    let info = InfoBuilder::new().title("Diapo API").version(crate::version::VERSION).build();
    OpenApiBuilder::new().info(info).paths(paths.build()).build()
}

/// The page's title, description and servers.
pub fn config() -> tronc::apiref::Config {
    tronc::apiref::Config {
        title: "Diapo API".to_owned(),
        version: crate::version::VERSION.trim_start_matches('v').to_owned(),
        description: "Slides as one text file: decks people edit on the web and agents edit over MCP.".to_owned(),
        servers: vec!["/api".to_owned()],
        script_url: String::new(),
    }
}
