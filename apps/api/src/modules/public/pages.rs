use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use super::info::{Shared, shared};
use super::thumbnail;
use crate::modules::share::service::url;
use crate::state::AppState;

/// `text` safe inside an HTML attribute or element.
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// `text` as one query parameter value.
pub fn encode(text: &str) -> String {
    let plain = |byte: u8| byte.is_ascii_alphanumeric() || b"-._~".contains(&byte);
    text.bytes().map(|byte| if plain(byte) { char::from(byte).to_string() } else { format!("%{byte:02X}") }).collect()
}

fn meta(property: &str, content: &str) -> String {
    format!("<meta property=\"{property}\" content=\"{}\">\n", escape(content))
}

async fn tags(state: &AppState, token: &str, deck: &Shared) -> String {
    let page = url(state, token);
    let mut head = [meta("og:title", &deck.title), meta("og:type", "website"), meta("og:url", &page)].concat();
    head.push_str(&meta("og:site_name", "Diapo"));
    if thumbnail::exists(state, deck.id, deck.rev).await {
        head.push_str(&meta("og:image", &format!("{}{}", state.public_url, thumbnail::route(token))));
    }
    let oembed = format!("{}/oembed?url={}&format=json", state.public_url, encode(&page));
    head.push_str(&format!(
        "<link rel=\"alternate\" type=\"application/json+oembed\" href=\"{}\" title=\"{}\">\n",
        escape(&oembed),
        escape(&deck.title)
    ));
    head
}

/// The client's page shell with `head` added to its `<head>`, or a bare document holding only
/// `head` when no client is built. The shell is the app's own HTML: `head` holds escaped text.
async fn shell(state: &AppState, status: StatusCode, head: &str) -> Response {
    let head = format!("<meta name=\"robots\" content=\"noindex\">\n{head}");
    let html = match tokio::fs::read_to_string(state.client_dir.join("index.html")).await {
        Ok(page) if page.contains("</head>") => page.replacen("</head>", &format!("{head}</head>"), 1),
        _ => format!(
            "<!doctype html>\n<html>\n<head>\n<meta charset=\"utf-8\">\n{head}</head>\n<body></body>\n</html>\n"
        ),
    };
    let headers = [
        (header::CONTENT_TYPE, HeaderValue::from_static("text/html; charset=utf-8")),
        (header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
    ];
    (status, headers, html).into_response()
}

/// `GET /p/{token}`: the public present page. The client draws it; the server adds `og:title`,
/// `og:image` once a thumbnail exists, and the oEmbed discovery link, for readers that run no
/// script. A link that is not live answers the bare shell with `404`.
pub async fn present(state: &AppState, token: &str) -> Response {
    match shared(state, token).await {
        Ok(deck) => shell(state, StatusCode::OK, &tags(state, token, &deck).await).await,
        Err(_) => shell(state, StatusCode::NOT_FOUND, "").await,
    }
}

/// `GET /embed/{token}`: the same viewer for another site to frame. A link that is not live
/// answers the bare shell with `404`.
pub async fn embed(state: &AppState, token: &str) -> Response {
    match shared(state, token).await {
        Ok(deck) => shell(state, StatusCode::OK, &format!("<title>{}</title>\n", escape(&deck.title))).await,
        Err(_) => shell(state, StatusCode::NOT_FOUND, "").await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_escaped_for_html_and_for_a_query() {
        assert_eq!(escape("a \"b\" <c> & d"), "a &quot;b&quot; &lt;c&gt; &amp; d");
        assert_eq!(encode("https://diapo.test/p/a-b_c"), "https%3A%2F%2Fdiapo.test%2Fp%2Fa-b_c");
        assert_eq!(
            meta("og:title", "\"><script>"),
            "<meta property=\"og:title\" content=\"&quot;&gt;&lt;script&gt;\">\n"
        );
    }
}
