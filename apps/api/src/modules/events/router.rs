use std::convert::Infallible;

use axum::Router;
use axum::extract::{Path, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::get;
use futures_util::stream::{self, Stream};
use porte::Identity;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::error::RecvError;
use tronc::Result;
use uuid::Uuid;

use super::Changed;
use crate::extract::id;
use crate::modules::decks::access::{self, Access};
use crate::modules::decks::service::fetch;
use crate::state::AppState;

struct Feed {
    state: AppState,
    user_id: i64,
    deck: Uuid,
    listener: Receiver<Changed>,
    first: Option<i64>,
}

fn changed(rev: i64) -> Event {
    Event::default().event("changed").data(format!("{{\"rev\":{rev}}}"))
}

async fn heard(feed: &mut Feed) -> Option<i64> {
    loop {
        match feed.listener.recv().await {
            Ok(event) if event.deck == feed.deck => return Some(event.rev),
            Ok(_) => {}
            Err(RecvError::Lagged(_)) => return fetch(&feed.state, feed.deck).await.ok().map(|row| row.rev),
            Err(RecvError::Closed) => return None,
        }
    }
}

async fn next(mut feed: Feed) -> Option<(std::result::Result<Event, Infallible>, Feed)> {
    if let Some(rev) = feed.first.take() {
        return Some((Ok(changed(rev)), feed));
    }
    let rev = heard(&mut feed).await?;
    let have = access::access(&feed.state, feed.user_id, feed.deck).await.ok()?;
    (have >= Access::Viewer).then(|| (Ok(changed(rev)), feed))
}

async fn events(
    State(state): State<AppState>,
    who: Identity,
    Path(raw): Path<String>,
) -> Result<Sse<impl Stream<Item = std::result::Result<Event, Infallible>>>> {
    let deck = id(&raw)?;
    access::check(&state, who.user_id, deck, Access::Viewer).await?;
    let listener = state.events.listen();
    let first = Some(fetch(&state, deck).await?.rev);
    let feed = Feed { state, user_id: who.user_id, deck, listener, first };
    Ok(Sse::new(stream::unfold(feed, next)).keep_alive(KeepAlive::default()))
}

/// `GET /decks/{id}/events`, behind `require_auth`: a server-sent stream of `changed` events,
/// each with `{"rev": n}`. The first one says where the deck is now, so a reader that loaded
/// the deck before it subscribed misses nothing. The stream ends when the reader loses access.
pub fn routes() -> Router<AppState> {
    Router::new().route("/decks/{id}/events", get(events))
}
