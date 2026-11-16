//! The sandboxed frame: its policy, no cookie, and the signed links its assets load through.

mod common;

use axum::body::Body;
use common::fixture;
use common::upload;
use http::{Request, header};

const SOURCE: &str = "# One\n\n![](assets/dot.png)\n\n---\n\n## Two\n\n<img src=\"missing.png\">\n\
<script>document.title = 'ran';</script>\n";

fn link(html: &str) -> String {
    let start = html.find("/api/decks/").unwrap();
    let end = start + html[start..].find('"').unwrap();
    html[start..end].replace("&amp;", "&")
}

#[tokio::test]
async fn the_frame_is_sandboxed_sets_no_cookie_and_is_the_only_html() {
    let Some(pool) = common::pool("diapo_frame_headers").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    let login = common::http::browser_login(&app.kits.sessions, ada.user_id).await;
    let frame = common::http::with_cookie(&app.router, "GET", &format!("/api/decks/{id}/frame"), &login).await;
    assert_eq!(frame.status, 200, "{}", frame.text());
    assert_eq!(frame.headers["content-type"], "text/html; charset=utf-8");
    let policy = frame.headers["content-security-policy"].to_str().unwrap();
    let directives: Vec<&str> = policy.split(';').map(str::trim).collect();
    assert_eq!(directives[0], "sandbox allow-scripts", "{policy}");
    assert!(directives.contains(&"default-src 'none'"), "{policy}");
    assert!(!policy.contains("allow-same-origin") && !policy.contains("connect-src"), "{policy}");
    assert_eq!(policy, api::modules::frame::POLICY);
    assert!(frame.headers.get(header::SET_COOKIE).is_none());
    assert_eq!(frame.headers["x-content-type-options"], "nosniff");
    assert_eq!(frame.headers["cache-control"], "no-store");
    assert!(frame.text().contains("document.title = 'ran';"));
    let deck = app.call("GET", &format!("/api/decks/{id}"), Some(&ada), None).await;
    assert_eq!(deck.headers["content-type"], "application/json");
    assert!(deck.headers.get(header::SET_COOKIE).is_none());
}

#[tokio::test]
async fn the_frame_loads_its_assets_through_signed_links_without_a_cookie() {
    let Some(pool) = common::pool("diapo_frame_assets").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    assert_eq!(upload::png(&app, &ada, &id, "assets/dot.png").await.status, 201);
    let html = app.call("GET", &format!("/api/decks/{id}/frame"), Some(&ada), None).await.text();
    let signed = link(&html);
    assert!(signed.starts_with(&format!("/api/decks/{id}/assets/assets/dot.png?frame=")), "{signed}");
    assert!(html.contains("src=\"missing.png\""), "an unknown asset keeps its path");
    let served = app.call("GET", &signed, None, None).await;
    assert_eq!((served.status.as_u16(), served.bytes.as_slice()), (200, upload::PNG));
    assert_eq!(served.headers["content-type"], "image/png");
    assert_eq!(served.headers["access-control-allow-origin"], "*");
    assert!(served.headers.get(header::SET_COOKIE).is_none());
    let bare = signed.split('?').next().unwrap().to_owned();
    assert_eq!(app.call("GET", &bare, None, None).await.status, 401);
    assert_eq!(app.call("GET", &format!("{bare}?frame=1.00"), None, None).await.status, 404);
    let other = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    upload::png(&app, &ada, &other, "assets/dot.png").await;
    let stolen = signed.replace(&id, &other);
    assert_eq!(app.call("GET", &stolen, None, None).await.status, 404);
    let expired = app.state.frame_key.token(uuid::Uuid::parse_str(&id).unwrap(), 1);
    assert_eq!(app.call("GET", &format!("{bare}?frame={expired}"), None, None).await.status, 404);
}

#[tokio::test]
async fn one_scene_compiles_alone_and_a_missing_one_is_not_found() {
    let Some(pool) = common::pool("diapo_frame_scene").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    let whole = app.call("GET", &format!("/api/decks/{id}/frame"), Some(&ada), None).await.text();
    assert!(whole.contains("One") && whole.contains("Two"));
    let second = app.call("GET", &format!("/api/decks/{id}/frame?scene=1"), Some(&ada), None).await;
    assert_eq!(second.headers["content-security-policy"], api::modules::frame::POLICY);
    assert!(second.text().contains("Two") && !second.text().contains("<h1>One"), "{}", second.text());
    assert_eq!(app.call("GET", &format!("/api/decks/{id}/frame?scene=2"), Some(&ada), None).await.status, 404);
    assert_eq!(app.call("GET", &format!("/api/decks/{id}/frame?scene=x"), Some(&ada), None).await.status, 400);
}

#[tokio::test]
async fn an_asset_answers_a_range_and_an_svg_is_sandboxed() {
    let Some(pool) = common::pool("diapo_frame_range").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let id = fixture::id(&fixture::deck(&app, &ada, None, SOURCE).await);
    let svg = b"<svg xmlns=\"http://www.w3.org/2000/svg\"><script>alert(1)</script></svg>";
    let file = upload::File { name: Some("logo.svg"), file_name: "logo.svg", bytes: svg };
    assert_eq!(upload::upload(&app, &ada, &id, file).await.status, 201);
    let request = Request::get(format!("/api/decks/{id}/assets/logo.svg"))
        .header(header::AUTHORIZATION, format!("Bearer {}", ada.token))
        .header(header::RANGE, "bytes=0-3")
        .body(Body::empty())
        .unwrap();
    let part = common::http::raw(&app.router, request).await;
    assert_eq!((part.status.as_u16(), part.bytes.as_slice()), (206, &b"<svg"[..]));
    assert_eq!(part.headers["content-type"], "image/svg+xml");
    assert!(part.headers["content-security-policy"].to_str().unwrap().starts_with("sandbox;"));
    assert_eq!(part.headers["x-content-type-options"], "nosniff");
    assert!(part.headers.get("access-control-allow-origin").is_none());
}
