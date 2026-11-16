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
client's `bun run dev` (in `apps/client`) proxies `/api` to it.

## The workspace

| Member | What it is |
|---|---|
| `crates/scene` | `facile-scene`: the format. No database, no network, no async |
| `apps/api` | The `api` binary: the server, `healthcheck`, `migrate` |

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
