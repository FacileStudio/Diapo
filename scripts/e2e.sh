#!/usr/bin/env sh
set -eu

cd "$(git rev-parse --show-toplevel)"

if [ -z "$(ls -d "${PLAYWRIGHT_BROWSERS_PATH:-$HOME/.cache/ms-playwright}"/chromium_headless_shell-* 2>/dev/null)" ]; then
  echo "e2e: no Playwright Chromium; install it with: cd apps/client && bunx playwright-core install chromium-headless-shell" >&2
  exit 2
fi

out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT

compile() {
  cargo run --quiet --locked -p facile-scene --example compile -- "$@"
}

compile crates/scene/fixtures/steps/diapo.md "$out/steps.html"
compile crates/scene/fixtures/steps/diapo.md "$out/steps-1.html" 1
compile crates/scene/fixtures/raw/clap.md "$out/clap.html"

cargo build --quiet --locked -p api

(
  cd apps/client
  [ -d node_modules ] || bun install --frozen-lockfile >/dev/null
  bun run build
  SCENE_E2E_DIR="$out" bun test --timeout 60000 ./e2e/scene.e2e.ts
  bun test --timeout 120000 ./e2e/app.e2e.ts
)
