-- The live Yjs state of a deck: the planche document its room edits. NULL until the deck is
-- first opened in a room or written as planche; `source` stays the projection every reader uses.
ALTER TABLE decks ADD COLUMN state bytea;
