# Diapo — Development

How to run Diapo locally, test it, and pass the gate before a commit.

## Prerequisites

- Rust 1.98.1 (`rust-toolchain.toml`) with `rustfmt` and `clippy`
- Docker, for Postgres 16, and `pg_isready` (psql 16 or later)
- bun 1.4 for the client, `filet` 0.21 or later, `mise` and `lefthook`
- Network access to GitHub: `tronc-rs` and `porte-rs` come from their public repos by tag

## Setup

```sh
mise trust && mise install
mise run db-up
```

`db-up` starts `postgres:16-alpine` as `diapo_dev_postgres` on `127.0.0.1:5673` (password
`postgres`) with `fsync` off, which is fine for a disposable local database. `mise run db-down`
stops it. Set `POSTGRES_PORT` to use another port.

## Run

```sh
mise run dev
```

The API listens on `:4000` with no Registre: sign-in routes answer `oidc_enabled: false`. The
client's `bun run dev` (in `apps/client`) proxies `/api` to it. Assets go under `DATA_DIR`,
which the task sets to `.local/data` (ignored by git). `/mcp` is off until
`MCP_RESOURCE` and `MCP_STATE_KEY` are set. The routes are in [api.md](api.md).

## The workspace

| Member | What it is |
|---|---|
| `crates/scene` | `facile-scene`: the format. No database, no network, no async |
| `apps/api` | The `api` binary: the server, `healthcheck`, `migrate` |
| `crates/capture` | `diapo-capture`: the capture job files, and with the `chrome` feature the browser client |
| `apps/worker` | The `worker` binary: the isolated capture service |
| `apps/cli` | `diapo-cli`: the `diapo` binary. Blocking I/O, no async runtime |

`apps/client` is the SvelteKit client. It is not a Cargo member.

## Tests

```sh
export TEST_DATABASE_URL='postgres://postgres:postgres@127.0.0.1:5673/postgres?sslmode=disable'
cargo test --workspace --locked
```

| Where | Covers |
|---|---|
| `crates/scene/tests/parse.rs` | Splitting, frontmatter, notes, steps and durations |
| `crates/scene/tests/compile.rs` | The compiled document, asset rewriting, the banned runtime calls |
| `crates/scene/tests/golden.rs` | Each fixture in `crates/scene/fixtures/` against its golden file |
| `apps/api/tests/shell.rs` | Probes, `/version`, sessions and the API reference, on a real Postgres |
| `apps/api/tests/decks.rs` | Create, read, list, the `rev` rule (`409`), invalid sources (`422`), revisions, the trash |
| `apps/api/tests/authorization.rs` | Every deck route against every role, cross-space `404`, the CSRF header |
| `apps/api/tests/frame.rs` | The frame's policy and missing cookie, signed asset links, ranges, SVG |
| `apps/api/tests/assets.rs` | Sniffing, names (traversal refused), caps, the sweep |
| `apps/api/tests/events.rs` | The server-sent `changed` stream |
| `apps/api/tests/directory.rs` | The directory snapshot, retired spaces, account erasure |
| `apps/api/tests/share.rs`, `public.rs` | Share links, the public routes (404 edges, no cookie, no notes), `og:` tags, oEmbed, the embed header |
| `apps/api/tests/exports.rs` | HTML and PPTX exports, signed downloads, the purge |
| `crates/scene/tests/pptx.rs` | The PPTX package: well-formed parts, text, lists, tables, pictures, notes. Opens it with LibreOffice when `soffice` is installed, and prints `SKIPPED` otherwise |
| `crates/scene/tests/pptx_tools.rs` | The PPTX read back by python-pptx (through `uv`, or a `python3` venv): slide count, titles, list levels, a table, pictures, notes. Prints `SKIPPED` when neither runs |
| `crates/capture/tests/shoot.rs` | Real captures in `chrome-headless-shell`: thumbnail, still, both PDFs, and a page that reaches nothing outside its bundle. Prints `SKIPPED` without Chrome |
| `apps/worker/src/**` | The worker's job loop, limits, cancel and link safety |
| `apps/api/tests/capture.rs` | PDF export, PPTX pictures and thumbnails against a worker that follows the file protocol |
| `apps/api/tests/clap.rs` | MP4 export against a mock of Clap's `POST /api/renders` |
| `apps/api/tests/mcp.rs`, `mcp_tools.rs` | tronc's `mcptest` conformance, the eight tools, scope gates, the delete confirmation |
| `apps/cli/tests/*.rs` | The HTTP module against an in-process mock server, the sync decisions, every command through the binary, and `live.rs`: push and pull against the real API on Postgres |
| `apps/client/src/**/*.test.ts` | The client's unit tests (vitest, jsdom) |

Without `TEST_DATABASE_URL` the database tests print a skip banner and pass: read the output.

After a deliberate change to the compiled output, run `UPDATE_GOLDEN=1 cargo test -p
facile-scene` and read the diff of `crates/scene/fixtures/` before committing it.

## End to end

```sh
mise run e2e
```

`scripts/e2e.sh` compiles the `steps` and `raw` fixtures with the `compile` example, then
`apps/client/e2e/scene.e2e.ts` opens them in headless Chromium through `playwright-core` and
drives `window.__scene`: `step(k)` reveals the markers, `seek(t)` maps time to scenes, a
timeline draws the same frame whatever was drawn before, and a single-scene document takes
`postMessage` commands.

Then `apps/client/e2e/app.e2e.ts` runs the app itself: `e2e/stack.ts` starts the `api` binary on
a scratch database (`diapo_e2e`, in the Postgres of `TEST_DATABASE_URL`) with the built client
and one signed-in person, and the test creates a deck, types a second slide, sees the preview
follow, goes through a refused source and a conflict, then steps through present mode in step
with the presenter view. `E2E_LOG=1` shows the API's request log.

It needs Playwright's Chromium headless shell under `~/.cache/ms-playwright`:
`cd apps/client && bunx playwright-core install chromium-headless-shell`. `check.sh` runs it
last and skips it with a message when the browser is missing or `SKIP_E2E` is set.

## The gate

```sh
sh scripts/check.sh
filet check .
```

`check.sh` runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked --
-D warnings` and `cargo test --workspace --locked`, then `bun run check`, `bun run test` and
`bun run build` in `apps/client`, then the end-to-end run when it can. `lefthook` runs it on
pre-push. Never raise a filet limit to pass.

Back to the [documentation index](README.md).
