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
    ("GET", "/auth/session", "auth", "The signed-in person, or {\"user\": null}, and bulle_url", "200"),
    ("POST", "/registre/sync", "registre", "Registre's directory nudge", "202"),
    ("POST", "/registre/deletions", "registre", "Registre's signed account deletion", "202"),
    ("GET", "/spaces", "spaces", "The caller's spaces and roles", "200"),
    ("GET", "/decks", "decks", "The decks the caller reaches, newest first (?space_id=&trashed=)", "200"),
    ("POST", "/decks", "decks", "Creates a deck from a source; 422 lists each invalid line", "201"),
    ("GET", "/decks/{id}", "decks", "One deck: metadata, source and rev", "200"),
    ("PATCH", "/decks/{id}", "decks", "Sets the title, the space or the trash", "200"),
    ("DELETE", "/decks/{id}", "decks", "Deletes a trashed deck for good", "204"),
    ("PUT", "/decks/{id}/source", "decks", "Saves {source, rev}; 409 on a stale rev, 422 on an invalid source", "200"),
    ("GET", "/decks/{id}/events", "decks", "Server-sent events: changed {rev}", "200"),
    ("GET", "/decks/{id}/frame", "frame", "The compiled HTML, sandboxed (?scene=n, from 0)", "200"),
    ("GET", "/decks/{id}/assets", "assets", "The assets of a deck", "200"),
    ("POST", "/decks/{id}/assets", "assets", "Uploads an asset (multipart `file`, optional `name`)", "201"),
    ("GET", "/decks/{id}/assets/{name}", "assets", "Serves an asset to a session or a frame link (?frame=)", "200"),
    ("DELETE", "/decks/{id}/assets/{name}", "assets", "Deletes an asset", "204"),
    ("POST", "/decks/{id}/share", "share", "Shares the deck; a new link answers its url once (?rotate=true)", "201"),
    ("GET", "/decks/{id}/share", "share", "Whether the deck is shared, and since when", "200"),
    ("DELETE", "/decks/{id}/share", "share", "Revokes the share link", "204"),
    ("GET", "/public/decks/{token}", "public", "A shared deck's title, scene count, steps and size; no session", "200"),
    ("GET", "/public/decks/{token}/frame", "public", "A shared deck's sandboxed frame (?scene=n); no session", "200"),
    ("GET", "/public/decks/{token}/assets/{name}", "public", "An asset of a shared deck; no session", "200"),
    ("GET", "/p/{token}", "public", "The public present page, with og: tags and the oEmbed link", "200"),
    ("GET", "/p/{token}/thumbnail.jpg", "public", "The first slide as a JPEG; 404 until one is rendered", "200"),
    ("GET", "/embed/{token}", "public", "The viewer for another site to frame", "200"),
    ("GET", "/oembed", "public", "oEmbed 1.0 for a live share URL (?url=&format=json)", "200"),
    (
        "POST",
        "/decks/{id}/exports",
        "exports",
        "Builds an export {format: html|pdf|pptx|mp4} and answers its job",
        "201",
    ),
    ("GET", "/exports/{id}", "exports", "An export job", "200"),
    (
        "GET",
        "/exports/{id}/file",
        "exports",
        "Downloads a finished export; a session or a signed link (?token=)",
        "200",
    ),
    ("GET", "/assistant", "assistant", "The caller's Bulle assistant name and accent", "200"),
    ("GET", "/mcp/pending", "mcp", "The person's pending agent actions, own browser session only", "200"),
    ("POST", "/mcp/pending/{id}/confirm", "mcp", "Confirms and runs a pending agent action", "200"),
    ("POST", "/mcp/pending/{id}/dismiss", "mcp", "Dismisses a pending agent action", "200"),
];

fn method(name: &str) -> HttpMethod {
    match name {
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "PATCH" => HttpMethod::Patch,
        "DELETE" => HttpMethod::Delete,
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
