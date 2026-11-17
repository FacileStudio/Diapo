//! What a room accepts: the checks every update passes, and the edits a whole new document needs.

use planche::schema::{self, Touched};
use planche::{Document, Edit, Profile};
use salon::Refusal;
use yrs::Doc;

use crate::modules::decks::error::DeckError;
use crate::modules::decks::source;

fn refusal(error: DeckError) -> Refusal {
    match error {
        DeckError::Api(error) if error.code == tronc::Code::ResourceExhausted => Refusal(1009, error.message),
        DeckError::Api(error) => Refusal(1008, error.message),
        DeckError::Invalid(errors) => Refusal(1008, errors.to_string()),
        DeckError::Stale { rev, .. } => Refusal(1008, format!("the deck moved on to rev {rev}")),
    }
}

/// Checks `doc` after a transaction that touched `touched`: the `full` profile, then the
/// projection, which must fit in `max` bytes and compile. Answers the projection.
pub fn check(doc: &Doc, touched: &Touched, max: usize) -> Result<source::Prepared, Refusal> {
    schema::check(doc, touched, Profile::Full).map_err(|reason| Refusal(1008, reason))?;
    let document = planche::yjs::read(doc).map_err(|error| Refusal(1008, error.0))?;
    source::prepared(document, max).map_err(refusal)
}

/// The edits that turn `current` into `next`: removed shapes, a changed `meta`, then every shape
/// added or changed, written whole.
pub fn changes(current: &Document, next: &Document) -> Vec<Edit> {
    let removed = current.shapes.keys().filter(|id| !next.shapes.contains_key(*id)).map(|id| Edit::Remove(id.clone()));
    let changed = next.shapes.iter().filter(|(id, shape)| current.shapes.get(*id) != Some(shape));
    let meta = (current.meta != next.meta).then_some(Edit::Meta(next.meta));
    removed.chain(meta).chain(changed.map(|(id, shape)| Edit::Insert(id.clone(), shape.clone()))).collect()
}
