#!/usr/bin/env sh
# The gate. Stages, in order: the Rust workspace (fmt, clippy -D warnings, tests) with the
# scene crate, the API and the `diapo` command line, the client (svelte-check, vitest, build),
# then the browser run of the compiled scenes.
#   SKIP_E2E=1          skips the browser run
#   TEST_DATABASE_URL   the Postgres the API tests use (default: the local container)
set -eu

cd "$(git rev-parse --show-toplevel)"

: "${TEST_DATABASE_URL:=postgres://postgres:postgres@127.0.0.1:${POSTGRES_PORT:-5673}/postgres?sslmode=disable}"
export TEST_DATABASE_URL
if ! pg_isready -d "$TEST_DATABASE_URL" -q 2>/dev/null; then
  echo "check: no Postgres at TEST_DATABASE_URL; the database tests will skip (run: mise run db-up)" >&2
fi

cli_version="v$(sed -n 's/^version = "\(.*\)"/\1/p' apps/cli/Cargo.toml | head -n 1)"
if [ "$cli_version" != "$(sh scripts/version.sh)" ]; then
  echo "check: apps/cli is $cli_version but apps/client/package.json is $(sh scripts/version.sh); a release tag must match both" >&2
  exit 1
fi

echo "==> crates/scene, apps/api, apps/cli"
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

echo "==> apps/client"
(
  cd apps/client
  [ -d node_modules ] || bun install --frozen-lockfile >/dev/null
  bun run check
  bun run test
  bun run build
)

echo "==> e2e"
if [ -n "${SKIP_E2E:-}" ]; then
  echo "check: SKIP_E2E is set, skipping the browser run" >&2
elif [ -z "$(ls -d "${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"/chromium_headless_shell-* 2>/dev/null)" ]; then
  echo "check: no Playwright Chromium, skipping the browser run (see docs/development.md#end-to-end)" >&2
else
  sh scripts/e2e.sh
fi
