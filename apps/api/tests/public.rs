//! What a share link opens: no session, no cookie, no source, no note, and 404 for anything else.

mod common;

use common::fixture;
use common::upload;
use http::header;
use serde_json::json;

use common::HOST;

fn routes(token: &str) -> Vec<String> {
    vec![
        format!("/api/public/decks/{token}"),
        format!("/api/public/decks/{token}/frame"),
        format!("/api/public/decks/{token}/frame?scene=1"),
        format!("/p/{token}"),
        format!("/embed/{token}"),
        format!("/oembed?url={HOST}/p/{token}&format=json"),
    ]
}

#[tokio::test]
async fn a_shared_deck_answers_its_facts_and_frame_without_source_notes_or_cookies() {
    let Some(pool) = common::pool("diapo_public").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let (id, token) = fixture::shared(&app, &ada, None).await;
    upload::png(&app, &ada, &id, "assets/dot.png").await;
    let login = common::http::browser_login(&app.kits.sessions, ada.user_id).await;
    for path in routes(&token) {
        let answer = common::http::with_cookie(&app.router, "GET", &path, &login).await;
        assert_eq!(answer.status, 200, "{path}: {}", answer.text());
        assert!(answer.headers.get(header::SET_COOKIE).is_none(), "{path}");
        let body = answer.text();
        assert!(!body.contains("SECRET") && !body.contains(&id), "{path}: {body}");
    }
    let facts = app.call("GET", &format!("/api/public/decks/{token}"), None, None).await.json();
    let expected = json!({"title": "Launch \"plan\" <b>", "scenes": 2, "steps": 3, "width": 1920, "height": 1080,
        "rev": 1, "frame": format!("/api/public/decks/{token}/frame"), "thumbnail": null});
    assert_eq!(facts, expected);
    let frame = app.call("GET", &format!("/api/public/decks/{token}/frame"), None, None).await;
    let policy = frame.headers["content-security-policy"].to_str().unwrap();
    assert!(policy.starts_with("sandbox allow-scripts; default-src 'none';") && !policy.contains("connect-src"));
    assert!(policy.ends_with("frame-ancestors 'self' https://prisme.facile.studio"), "{policy}");
    let html = frame.text();
    let link = format!("/api/public/decks/{token}/assets/assets/dot.png");
    assert!(html.contains(&format!("src=\"{link}\"")), "{html}");
    let asset = app.call("GET", &link, None, None).await;
    assert_eq!((asset.status.as_u16(), asset.bytes.as_slice()), (200, upload::PNG));
    assert!(asset.headers.get(header::SET_COOKIE).is_none() && asset.headers["access-control-allow-origin"] == "*");
    let one = app.call("GET", &format!("/api/public/decks/{token}/frame?scene=1"), None, None).await.text();
    assert!(one.contains("Next") && !one.contains("<h1>Launch"), "{one}");
    assert_eq!(app.call("GET", &format!("/api/public/decks/{token}/frame?scene=9"), None, None).await.status, 404);
}

#[tokio::test]
async fn an_unshared_unknown_or_revoked_link_is_not_found_everywhere() {
    let Some(pool) = common::pool("diapo_public_404").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let private = fixture::id(&fixture::deck(&app, &ada, None, fixture::NOTED).await);
    let unknown = porte::new_token().unwrap();
    for token in [unknown.as_str(), private.as_str(), "..", "a"] {
        let more = [format!("/p/{token}/thumbnail.jpg"), format!("/api/public/decks/{token}/assets/assets/dot.png")];
        for path in routes(token).into_iter().chain(more) {
            let answer = app.call("GET", &path, None, None).await;
            assert_eq!(answer.status, 404, "{path}");
            assert!(!answer.text().contains("Launch"), "{path}: {}", answer.text());
        }
    }
    let (id, token) = fixture::shared(&app, &ada, None).await;
    for url in [
        format!("{HOST}/decks/{id}"),
        format!("{HOST}/decks/{private}"),
        format!("https://evil.test/p/{token}"),
        format!("{HOST}/embed/{token}"),
        format!("{HOST}/p/{token}/thumbnail.jpg"),
    ] {
        assert_eq!(app.call("GET", &format!("/oembed?url={url}"), None, None).await.status, 404, "{url}");
    }
    assert_eq!(app.call("GET", "/oembed", None, None).await.status, 404);
    assert_eq!(app.call("GET", &format!("/oembed?url={HOST}/p/{token}&format=xml"), None, None).await.status, 501);
    assert_eq!(app.call("DELETE", &format!("/api/decks/{id}/share"), Some(&ada), None).await.status, 204);
    for path in routes(&token) {
        assert_eq!(app.call("GET", &path, None, None).await.status, 404, "revoked {path}");
    }
}

#[tokio::test]
async fn the_page_and_oembed_describe_the_deck_without_a_picture_while_none_exists() {
    let Some(pool) = common::pool("diapo_public_card").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let (_, token) = fixture::shared(&app, &ada, None).await;
    let page = app.call("GET", &format!("/p/{token}"), None, None).await.text();
    assert!(page.contains("<div id=\"app\"></div>"), "{page}");
    assert!(page.contains("<meta property=\"og:title\" content=\"Launch &quot;plan&quot; &lt;b&gt;\">"), "{page}");
    assert!(page.contains(&format!("<meta property=\"og:url\" content=\"{HOST}/p/{token}\">")), "{page}");
    assert!(page.contains("application/json+oembed") && !page.contains("og:image"), "{page}");
    assert_eq!(app.call("GET", &format!("/p/{token}/thumbnail.jpg"), None, None).await.status, 404);
    let card = app.call("GET", &format!("/oembed?url={HOST}/p/{token}&format=json"), None, None).await.json();
    let iframe = format!(
        "<iframe src=\"{HOST}/embed/{token}\" width=\"960\" height=\"540\" title=\"Launch &quot;plan&quot; \
        &lt;b&gt;\" frameborder=\"0\" allowfullscreen></iframe>"
    );
    let expected = json!({"version": "1.0", "type": "rich", "provider_name": "Diapo", "provider_url": HOST,
        "title": "Launch \"plan\" <b>", "html": iframe, "width": 960, "height": 540});
    assert_eq!(card, expected);
}

#[tokio::test]
async fn a_thumbnail_file_turns_on_the_picture_route_og_image_and_the_oembed_fields() {
    let Some(pool) = common::pool("diapo_public_thumb").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let (id, token) = fixture::shared(&app, &ada, None).await;
    let picture = format!("/p/{token}/thumbnail.jpg");
    let missing = app.call("GET", &picture, None, None).await;
    assert_eq!((missing.status.as_u16(), missing.headers["cache-control"].to_str().unwrap()), (404, "no-store"));
    let file = api::modules::public::thumbnail::path(&app.state.data_dir, id.parse().unwrap(), 1);
    std::fs::create_dir_all(file.parent().unwrap()).unwrap();
    std::fs::write(&file, b"\xff\xd8\xff-jpeg").unwrap();
    let served = app.call("GET", &picture, None, None).await;
    assert_eq!((served.status.as_u16(), served.headers["content-type"].to_str().unwrap()), (200, "image/jpeg"));
    let page = app.call("GET", &format!("/p/{token}"), None, None).await.text();
    assert!(page.contains(&format!("<meta property=\"og:image\" content=\"{HOST}{picture}\">")), "{page}");
    let card = app.call("GET", &format!("/oembed?url={HOST}/p/{token}&maxwidth=480"), None, None).await.json();
    assert_eq!((card["thumbnail_url"].clone(), card["width"].clone()), (json!(format!("{HOST}{picture}")), json!(480)));
    assert_eq!((card["thumbnail_width"].clone(), card["thumbnail_height"].clone()), (json!(1280), json!(720)));
    let save = json!({"source": "# Changed\n", "rev": 1});
    assert_eq!(app.call("PUT", &format!("/api/decks/{id}/source"), Some(&ada), Some(save)).await.status, 200);
    assert_eq!(app.call("GET", &picture, None, None).await.status, 404);
}

#[tokio::test]
async fn only_the_embed_page_and_the_shared_frame_may_be_framed_by_prisme() {
    let Some(pool) = common::pool("diapo_public_embed").await else { return };
    let app = common::app(pool).await;
    let ada = app.person("ada").await;
    let (_, token) = fixture::shared(&app, &ada, None).await;
    let security = api::security::config(Default::default(), &app.state.client_dir, "https://bulle.test");
    let layers = tronc::http::layers(tronc::http::Config { security, ..Default::default() }).unwrap();
    let router = app.router.clone().layer(layers);
    let get = |path: String| common::send(&router, "GET", Box::leak(path.into_boxed_str()), None, None);
    let framing = |answer: &common::Answer| {
        let policy = answer.headers["content-security-policy"].to_str().unwrap().to_owned();
        policy.split(';').map(str::trim).find(|part| part.starts_with("frame-ancestors")).unwrap().to_owned()
    };
    let embed = get(format!("/embed/{token}")).await;
    assert_eq!(embed.status, 200);
    assert_eq!(framing(&embed), "frame-ancestors 'self' https://prisme.facile.studio");
    assert!(embed.headers.get("x-frame-options").is_none());
    assert!(embed.headers.get(header::SET_COOKIE).is_none());
    let frame = get(format!("/api/public/decks/{token}/frame")).await;
    assert_eq!(framing(&frame), "frame-ancestors 'self' https://prisme.facile.studio");
    assert!(frame.headers.get("x-frame-options").is_none());
    assert!(frame.headers["content-security-policy"].to_str().unwrap().starts_with("sandbox allow-scripts;"));
    let page = get(format!("/p/{token}")).await;
    assert_eq!(
        (framing(&page).as_str(), page.headers["x-frame-options"].to_str().unwrap()),
        ("frame-ancestors 'none'", "DENY")
    );
}
