# Diapo — Architecture

How a deck is stored, edited live and read. The routes and their bodies are in
[api.md](api.md); the deck format is in [planche.md](planche.md).

## One deck, two columns

| Column | Holds | Written by |
|---|---|---|
| `decks.source` | The planche projection: the one text every reader compiles | every write |
| `decks.state` | The Yjs state of the same document, a full v1 update. `NULL` until a room opens the deck | the room, and its first load |
| `decks.rev` | The revision every write names | every write that changes the source |
| `deck_revisions` | Every stored source, with its author and origin | every write |

`source` and `state` always hold the same document once `state` is set: only the room writes
`state`, and every write to a deck that has a `state` goes through the room. A deck written
before planche (migration `0007_deck_state.sql` added `state`) keeps its scene markdown until
its first write or room load converts it; every reader goes through `scene::planche::project`,
which compiles both formats.

## The crates

- `facile-scene` with its `planche` feature (`crates/scene/src/planche/`) is the only bridge
  between the scene format and planche: `convert` reads either format into a planche document,
  `project` compiles either format into a scene `Project`, `STARTER` and `GUIDE`.
- `planche` v0.3.0 is the document: schema, profiles, the Yjs reader and writer, the projection.
- `salon` v0.1.0 hosts the rooms: the hub, the room loop, y-sync, presence, slow members,
  socket re-checks, shutdown. Diapo implements its two hooks in `apps/api/src/modules/rooms/`.

## Writes

```
REST PUT /decks/{id}/source ─┐
MCP diapo_write_deck ────────┼─> decks::write::save ─> convert, project, cap
MCP diapo_edit_deck ─────────┘        │
                                      ├─ state IS NULL: UPDATE source, rev + 1, revision
                                      └─ state set: rooms::Hub::save ─> the room (opened if
                                         needed) checks rev, applies the diff as one
                                         transaction, relays it, stores it
y-websocket update ─> rooms::Live::apply ─> check ─> relay ─> save 2 s after the last update
```

Every path ends in the same transaction shape: `state`, `source`, `rev + 1`, `updated_at`,
`updated_by` and a `deck_revisions` row, then the in-process `changed` event, which the SSE
route, the frame previews and the thumbnail maker follow.

A REST or MCP write to a deck with a `state` always runs in its room, even when no editor is
connected: the room is the only writer of `state`, so a direct write could never race a room
that is loading or holds unsaved edits.

## The room

`modules/rooms/`:

| File | Does |
|---|---|
| `space.rs` | `Live`, salon's `Space`: the Yjs document, its last accepted state, the stored `rev`. Each update is applied, checked (planche `full` on the touched shapes, then the projection's size and compile), and undone exactly when refused |
| `load.rs` | Opens a deck: `state`, or `source` converted and stored at once |
| `persist.rs` | The two store transactions: the first conversion, and a save |
| `host.rs` | salon's `Host` on `AppState`: the policy (2 s debounce, 60 s idle, blocking work), load, save, the socket re-check and the access re-check |
| `hub.rs` | The calls the rest of the API makes: `save` (a REST or MCP write), `close` (moved, trashed, restored), `delete`, `evict`, `shutdown` |
| `router.rs` | `GET /decks/{id}/ws`: origin check, access, then salon's accept |
| `live.rs` | Whether a socket's session still stands |

Who may stay: every 30 seconds a socket re-checks its session. A directory sync that removes a
membership or changes a role re-checks those people in every open room, and closes the rooms of
the decks it trashed. A move, a trash or a restore closes the deck's room with `1001`, so its
members reconnect under the new rules.

The hub is in-process, so the app runs as one replica (`docker-compose.yml` pins it).

## Auth

`apps/api/src/auth` wires porte: sessions and cookies, Registre sign-in, the directory sync into
`spaces` and `space_members`, account deletions, token exchange and the MCP verifier. One
`access()` in `modules/decks/access.rs` authorizes every deck query, the room included.

Back to the [documentation index](README.md).
