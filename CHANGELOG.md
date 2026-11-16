# Changelog

All notable changes to Diapo. Versions follow SemVer.

## [Unreleased]

## [0.2.0] - 2026-11-16

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
- The web client: the deck list with first-slide thumbnails, create, rename and trash; the
  split editor at `/decks/{id}` (CodeMirror source with autosave, error lines from a `422`, a
  conflict banner on a `409`, live reload on `changed`, live preview following the caret, scene
  strip, assets panel, insert menu, Bulle sidebar); present mode at `/decks/{id}/present` and
  the presenter view at `/decks/{id}/presenter`, kept in step through a `BroadcastChannel`; the
  trash, the settings page and the agent-request page at `/pending`. In French, English,
  Spanish and German.
- The public landing page at `/` for signed-out visitors: a mise-en-scene demo of the editor,
  four feature tiles, the suite pricing block, and a `/downloads` page for the `diapo` command.
  Favicons, the link preview image and the web manifest come from muse's brand generator. The
  Vision tracker loads on the two public pages as a pinned same-origin copy.
- Language is now picked on the settings page (muse v0.30.0 slimmed the user menu).
- The share dialog in the editor: create the link (shown once, with its embed code), generate a
  new one, turn it off. The public pages `/p/{token}` and `/embed/{token}` draw the shared deck
  without a session and call only `/api/public`; a dead link gets its own page.
- The export dialog in the editor: PowerPoint and HTML. A format the API answers `503` for is
  shown as coming soon, and is asked again until it works.
- The app mark is the Solar `presentation-graph-bold-duotone` glyph.
- Share links: `POST`, `GET` and `DELETE /api/decks/{id}/share`. One live link per deck; the
  token is shown once and stored only as its SHA-256.
- Public, cookie-less routes for a shared deck: `GET /api/public/decks/{token}` (no source, no
  notes), its sandboxed frame, the `/p/{token}` page with `og:` tags, `/embed/{token}` (framable
  by `EMBED_ANCESTORS`), `/oembed` and `/p/{token}/thumbnail.jpg` (404 until thumbnails are
  rendered).
- Exports: `POST /api/decks/{id}/exports`, `GET /api/exports/{id}` and a download route. `html`
  is one offline file, `pptx` an editable PowerPoint file; `pdf` and `mp4` answer `unavailable`.
  `diapo_export` builds `html` and `pptx` and answers a signed link valid one hour.
- `facile-scene`: the `pptx` feature, `scene::pptx::write`. Titles, text, leveled lists, tables,
  embedded pictures and presenter notes become editable shapes; a `raw` scene becomes a slide
  that says it is animated HTML.
- Migration `0004_share_exports.sql`. New variables: `PUBLIC_URL`, `EMBED_ANCESTORS`.
- New variables: `MCP_RESOURCE`, `MCP_STATE_KEY`, `SOURCE_MAX_BYTES`, `ASSET_MAX_BYTES`,
  `DECK_ASSETS_MAX_BYTES`, `BULLE_URL`, `BULLE_CLIENT_ID`.
- `apps/cli`: the `diapo` command line. `new`, `serve` (live reload, problems in the terminal
  and as a browser overlay), `build` (one offline HTML file), `check`, `guide`, `login`,
  `logout`, `whoami`, `list`, `pull`, `push`, `open`, and an `export` that says what is not
  built yet. `--json` on every command. Reference: `docs/cli.md`.
- The release workflow: a `v*` tag publishes `diapo` for Linux (static) and macOS, amd64 and
  arm64, with `checksums.txt`. `install.sh` installs it through `facile`.

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
