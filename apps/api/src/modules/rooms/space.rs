//! A room's deck: the planche Yjs document, checked after every update, and the stored state it
//! falls back to when an update is refused.

use std::sync::{Arc, Mutex};

use planche::Document;
use planche::schema::{self, Touched};
use salon::{Refusal, Space};
use yrs::updates::decoder::Decode;
use yrs::{Doc, Transact, Update};

use super::check::{changes, check};
use crate::modules::decks::types::Origin;

/// The deck of one room.
pub struct Live {
    /// The live document.
    pub doc: Doc,
    touched: Arc<Mutex<Touched>>,
    stable: Vec<u8>,
    /// The stored revision, as of the last save.
    pub rev: i64,
    /// A deck in the trash makes everyone a viewer.
    pub trashed: bool,
    /// The source cap, in bytes: the projection must fit in it.
    pub max: usize,
    /// The door of the change the next save stores; `None` is the web editor.
    pub origin: Option<Origin>,
}

impl Default for Live {
    fn default() -> Self {
        Live::new(planche::doc::new_doc(), 0, false, 0)
    }
}

impl Live {
    /// The room deck of `doc`, stored at `rev`, whose projection must fit in `max` bytes.
    pub fn new(doc: Doc, rev: i64, trashed: bool, max: usize) -> Live {
        let touched = schema::watch(&doc);
        let stable = planche::doc::encode(&doc);
        Live { doc, touched, stable, rev, trashed, max, origin: None }
    }

    fn settle<T>(&mut self, outcome: Result<T, Refusal>) -> Result<T, Refusal> {
        let touched = std::mem::take(&mut *self.touched.lock().unwrap_or_else(|poisoned| poisoned.into_inner()));
        match outcome.and_then(|kept| check(&self.doc, &touched, self.max).map(|_| kept)) {
            Ok(kept) => {
                self.stable = planche::doc::encode(&self.doc);
                Ok(kept)
            }
            Err(refused) => self.recover().and(Err(refused)),
        }
    }

    /// Makes the document equal `next` in one transaction that touches only the shapes that
    /// differ, checked like a member's update. Answers the update to relay.
    pub fn replace(&mut self, next: &Document) -> Result<Vec<u8>, Refusal> {
        let current = planche::yjs::read(&self.doc).map_err(|error| Refusal(1008, error.0))?;
        let edits = changes(&current, next);
        let outcome = {
            let mut txn = self.doc.transact_mut();
            planche::yjs::apply(&mut txn, &edits).map(|()| txn.encode_update_v1()).map_err(|e| Refusal(1008, e.0))
        };
        self.settle(outcome)
    }
}

impl Space for Live {
    fn doc(&self) -> &Doc {
        &self.doc
    }

    fn apply(&mut self, update: &[u8]) -> Result<bool, Refusal> {
        if update.len() > planche::MAX_UPDATE {
            return Err(Refusal(1009, "update over 1 MiB".to_owned()));
        }
        let decoded = Update::decode_v1(update).map_err(|e| Refusal(1008, format!("undecodable update: {e}")))?;
        if decoded.is_empty() {
            return Ok(false);
        }
        let applied = self.doc.transact_mut().apply_update(decoded);
        self.settle(applied.map_err(|e| Refusal(1008, format!("update refused: {e}")))).map(|()| true)
    }

    fn recover(&mut self) -> Result<(), Refusal> {
        let doc = planche::doc::from_state(&self.stable, std::iter::empty())
            .map_err(|e| Refusal(1011, format!("revert failed: {e}")))?;
        self.touched = schema::watch(&doc);
        self.doc = doc;
        Ok(())
    }

    fn writable(&self) -> bool {
        !self.trashed
    }
}
