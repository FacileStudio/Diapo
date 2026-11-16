use serde::{Deserialize, Serialize};
use tronc::{Error, Result};

use super::info::shared;
use super::pages::escape;
use super::thumbnail;
use crate::state::AppState;

/// The width of the embed when the consumer names none.
pub const WIDTH: u32 = 960;

/// `?url=&format=&maxwidth=&maxheight=`.
#[derive(Debug, Default, Deserialize)]
pub struct Query {
    pub url: Option<String>,
    pub format: Option<String>,
    pub maxwidth: Option<u32>,
    pub maxheight: Option<u32>,
}

/// An oEmbed 1.0 `rich` answer.
#[derive(Debug, Serialize)]
pub struct Embed {
    pub version: &'static str,
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub provider_name: &'static str,
    pub provider_url: String,
    pub title: String,
    pub html: String,
    pub width: u32,
    pub height: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail_height: Option<u32>,
}

/// The token of a share URL of this host, `<public url>/p/<token>`. Anything else names nothing:
/// another host, a private deck URL, a path below the page.
pub fn token<'a>(public_url: &str, url: &'a str) -> Option<&'a str> {
    let rest = url.strip_prefix(public_url)?.strip_prefix("/p/")?;
    let token = rest.split(['?', '#']).next()?.trim_end_matches('/');
    let plain = token.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
    (!token.is_empty() && plain).then_some(token)
}

/// The embed size for a stage of `stage`: [`WIDTH`] wide at the stage's ratio, shrunk to fit
/// `maxwidth` and `maxheight`.
pub fn size(stage: (u32, u32), maxwidth: Option<u32>, maxheight: Option<u32>) -> (u32, u32) {
    let (stage_width, stage_height) = (u64::from(stage.0.max(1)), u64::from(stage.1.max(1)));
    let mut width = u64::from(maxwidth.unwrap_or(WIDTH).clamp(1, WIDTH));
    let mut height = (width * stage_height / stage_width).max(1);
    if let Some(cap) = maxheight.map(u64::from).filter(|cap| *cap >= 1 && *cap < height) {
        height = cap;
        width = (height * stage_width / stage_height).max(1);
    }
    (width as u32, height as u32)
}

/// `GET /oembed`: the card of a live share URL of this host, and `404` for anything else, so a
/// private deck never shows that it exists.
pub async fn embed(state: &AppState, query: &Query) -> Result<Embed> {
    let missing = || Error::not_found("not found");
    let token = query.url.as_deref().and_then(|url| token(&state.public_url, url)).ok_or_else(missing)?;
    let deck = shared(state, token).await.map_err(|_| missing())?;
    let stage = deck.project.config().size;
    let (width, height) = size((stage.width, stage.height), query.maxwidth, query.maxheight);
    let pictured = thumbnail::exists(state, deck.id, deck.rev).await;
    let html = format!(
        "<iframe src=\"{}/embed/{token}\" width=\"{width}\" height=\"{height}\" title=\"{}\" frameborder=\"0\" \
         allowfullscreen></iframe>",
        state.public_url,
        escape(&deck.title)
    );
    Ok(Embed {
        version: "1.0",
        kind: "rich",
        provider_name: "Diapo",
        provider_url: state.public_url.to_string(),
        title: deck.title,
        html,
        width,
        height,
        thumbnail_url: pictured.then(|| format!("{}{}", state.public_url, thumbnail::route(token))),
        thumbnail_width: pictured.then_some(thumbnail::WIDTH),
        thumbnail_height: pictured.then(|| thumbnail::height(stage.width, stage.height)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_share_url_of_this_host_names_a_token() {
        let host = "https://diapo.test";
        assert_eq!(token(host, "https://diapo.test/p/abc-DEF_1"), Some("abc-DEF_1"));
        assert_eq!(token(host, "https://diapo.test/p/abc/?x=1#y"), Some("abc"));
        for url in [
            "https://evil.test/p/abc",
            "https://diapo.test.evil.test/p/abc",
            "https://diapo.test/decks/6f1e2d3c-4b5a-4968-8776-655443322110",
            "https://diapo.test/p/",
            "https://diapo.test/p/abc/thumbnail.jpg",
            "https://diapo.test/embed/abc",
            "/p/abc",
        ] {
            assert_eq!(token(host, url), None, "{url}");
        }
    }

    #[test]
    fn the_embed_keeps_the_stage_ratio_within_the_caps() {
        assert_eq!(size((1920, 1080), None, None), (960, 540));
        assert_eq!(size((1024, 768), None, None), (960, 720));
        assert_eq!(size((1920, 1080), Some(480), None), (480, 270));
        assert_eq!(size((1920, 1080), Some(4000), Some(270)), (480, 270));
        assert_eq!(size((1920, 1080), Some(0), Some(0)), (1, 1));
    }
}
