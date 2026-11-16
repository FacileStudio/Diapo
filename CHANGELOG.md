# Changelog

All notable changes to Diapo. Versions follow SemVer.

## [Unreleased]

### Added

- Decks: `GET/POST /api/decks`, `GET/PATCH/DELETE /api/decks/{id}` and `PUT
  /api/decks/{id}/source`. Every save names the `rev` it read; a stale one answers `409` with
  the current `rev` and changes nothing. A source that does not parse answers `422` with the
  line of each problem. Every saved source is kept in `deck_revisions` with its author and
  origin (`web`, `cli` or `mcp`).
- `GET /api/decks/{id}/events`: server-sent `changed {rev}` events for open editors.
- Assets: multipart upload, list, read and delete under `/api/decks/{id}/assets`. Files are
  typed by their bytes, named by safe relative paths, capped per file and per deck, and stored
  once by content under `DATA_DIR`.
- `GET /api/decks/{id}/frame`: the compiled deck, served with a `sandbox` policy and no cookie.
  It loads its assets through signed links valid one hour for one deck.
- The MCP server at `/mcp` (scopes `diapo.read`, `diapo.write`): `diapo_list_decks`,
  `diapo_get_deck`, `diapo_create_deck`, `diapo_write_deck`, `diapo_edit_deck`,
  `diapo_format_guide`, `diapo_export` (answers "not available yet") and `diapo_delete_deck`,
  which the person confirms.
- Spaces mirrored from Registre (`/api/registre/sync`), account erasure
  (`/api/registre/deletions`), token exchange, `GET /api/spaces` and `GET /api/assistant`.
- Migrations `0002_diapo.sql` (spaces, decks, revisions, assets) and `0003_mcp_actions.sql`.
- `docs/api.md`: the routes, the `rev` rule, the frame's security model and the MCP tools.
- New variables: `MCP_RESOURCE`, `MCP_STATE_KEY`, `SOURCE_MAX_BYTES`, `ASSET_MAX_BYTES`,
  `DECK_ASSETS_MAX_BYTES`, `BULLE_URL`, `BULLE_CLIENT_ID`.

## [0.1.0] - 2026-11-16

### Added

- `facile-scene` (`crates/scene`): the scene format shared with Clap. It parses one text file
  into scenes, reports every problem with its line, and compiles a self-contained HTML document
  with the `window.__scene` contract, step reveals, clips and a seekable timeline.
- The format reference, `docs/format.md`, also exported as `scene::FORMAT_GUIDE`.
- `apps/api`: the Rust API on tronc-rs and porte-rs. Sign-in through Registre, health routes,
  `/version`, the API reference at `/docs`, the SPA fallback, `healthcheck` and `migrate`.
- `apps/client`: the SvelteKit shell on muse. A sign-in screen, and an empty deck list once
  signed in.
- The gate (`scripts/check.sh`), filet, CI, the Dockerfile and the compose file.
