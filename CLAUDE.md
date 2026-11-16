# Diapo

Slides as one text file for the Facile suite, at `diapo.facile.studio`. A Cargo workspace: the
scene format in `crates/scene`, the Rust API in `apps/api`, the SvelteKit client in
`apps/client`, one container. The roadmap is `PLAN.md`.

## Stack

- **Format** (`crates/scene`, package `facile-scene`, lib `scene`): parser, validator and HTML
  compiler on pulldown-cmark 0.13. The runtime is plain JavaScript in `src/runtime/`, inlined
  into every compiled document. Clap pins this crate by git tag
- **API** (`apps/api`): Rust 1.98 (edition 2024), axum 0.8, sqlx 0.9 on Postgres 16
- **Libraries**: `tronc` (tronc-rs v0.5.1) and `porte` (porte-rs v0.8.1), from their public
  `github.com/FacileStudio` repos by tag (bump: `docs/deployment.md`). Never copy their code
  here; report a gap instead
- **Client** (`apps/client`): SvelteKit 2, Svelte 5 runes, Tailwind 4, `@facile/muse`

## Commands

```sh
mise run db-up              # Postgres 16 on 127.0.0.1:5673 (container diapo_dev_postgres)
sh scripts/check.sh         # the gate: fmt, clippy -D warnings, tests, the client, then e2e
mise run e2e                # the compiled scene fixtures in headless Chromium
filet check .               # after the gate, before every commit
mise run dev                # the API on :4000 against the local database
cargo run -p api -- migrate status
cargo run -p facile-scene --example compile -- deck.md deck.html
UPDATE_GOLDEN=1 cargo test -p facile-scene   # rewrite the golden files, then read the diff
```

The database tests read `TEST_DATABASE_URL` (check.sh defaults it to the local container) and
print a skip banner without it: read the output before trusting a green run.

## Invariants

- **The format is a contract.** Clap pins `facile-scene` by tag. Ask before any change to the
  format, to `window.__scene` or to the public API of the crate after `v0.1.0`. `docs/format.md`
  is the reference and ships in the crate: change it in the same commit as the behaviour.
- **A `Project` is valid by construction.** `scene::parse` is the only way to make one, and it
  answers every problem with its line. `compile` cannot fail.
- **The runtime has no clock.** Nothing in `src/runtime/` reads time, randomness or the
  network; a frame depends only on `seek(t)` or `step(k)`. A test bans the calls.
- **Compiled documents run author scripts.** Serve them only in a sandboxed frame
  (`sandbox="allow-scripts"`, no `allow-same-origin`), never in the app origin.
  `GET /api/decks/{id}/frame` is the only route that answers user HTML as HTML; its policy keeps
  `sandbox` and has no `connect-src`, and it never sets a cookie (`docs/api.md`).
- **One `access()` authorizes every deck query** (`modules/decks/access.rs`). `None` is `404`.
- **Every write names its `rev`** and goes through `modules::decks::write::save`, REST and MCP
  alike. A stale `rev` is `409` and changes nothing.
- **The MCP tool names are a contract.** Bulle calls the eight `diapo_*` tools by name.
- **Auth is porte's.** `apps/api/src/auth` wires porte as its `docs/architecture.md` says.
  Never hand-roll a session, a cookie or a token check.
- **Migrations** are append-only `NNNN_name.sql`; `0001_porte.sql` is `porte::pg::SCHEMA`
  verbatim, pinned by a test.
- **One replica.** `docker-compose.yml` pins `replicas: 1`.
- **No borders** in the UI or in the theme. Separate with fills and spacing.
- No inline comments, no comments in function bodies; doc comments on public items.
