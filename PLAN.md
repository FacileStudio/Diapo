# Diapo plan

Written 2026-11-16. Companion plans: `PLAN-suite-wiring.md` (this repo) and `PLAN.md` in
`FacileStudio/Clap`. Research behind the choices: wiki page
`syntheses/video-and-slides-as-code`.

## Goal

Ship Diapo at `diapo.facile.studio`: slides written as one text file that people edit on the
web, agents edit through MCP and Bulle, and the `diapo` CLI serves, builds and syncs.

## Why

The suite has Docs, Feuilles and Toile but no slides. The owner wants a deck to be a single
file in a high-level language, so that a person, Bulle or a coding agent can all edit it, from
the web and from a terminal. `~/Code/Saravenpi/Pres` (Go, personal repo) proved the folder and
markdown model; Diapo is its Rust successor inside the suite.

## Checked against

Read on 2026-11-16: `conventions/facile-suite-rules`, `syntheses/suite-deployment-design`,
`conventions/agent-access-facile-apps`, headings of `standards/cli`, `tools/dokploy` (hot
spots, DNS, Registre seed trap in `projects/registre`), and on `origin/main` of
`FacileStudio/Docs`: `CLAUDE.md`, `apps/api/Cargo.toml`, `Dockerfile`, `docker-compose.yml`,
`scripts/check.sh`, `mise.toml`, `filet.yml`, `docs/deployment.md`,
`apps/client/src/lib/bulle.ts`, `apps/api/src/modules/assistant/router.rs`. Also the READMEs
of `tronc-rs` (v0.5.2) and `porte-rs` (v0.8.1).

Not read in full: `standards/migrations`, `standards/cli`, `conventions/facile-idp-registre`,
`conventions/suite-contracts`, `conventions/facile-test-layout`. The agent that owns a step
flagged with one of them reads that page first.

## Owner decisions (2026-11-16)

- Rust backend on `tronc-rs` and `porte-rs`, SvelteKit client on muse, Rust CLI, mise, filet.
- Free app. Every AI feature goes through Bulle and needs `ai.access`. MCP is free.
- v1 exports: PDF, standalone HTML, editable PPTX, MP4 through Clap.
- One format shared with Clap.
- Public repo, AGPL-3.0-only, like Docs.

## The scene format (shared with Clap)

A project is a folder: one entry file (`diapo.md` or `clap.md`) and its assets beside it.
The conventions are Slidev's, so they are already familiar
(https://sli.dev/guide/syntax).

```md
---
title: Q3 review
theme: facile
size: 1920x1080
---

# Q3 review

Speaker text here.

<!-- Presenter note: thank the team first. -->

---
layout: split
steps: 2
---

## Revenue

- Up 12% {step=1}
- Churn down {step=2}

![](assets/chart.png)

---
layout: raw
duration: 6
---

<div class="clip" data-start="0" data-duration="6">
  <h1 id="t">Launch day</h1>
</div>
<script>
  scene.timeline(tl => tl.from('#t', { opacity: 0, y: 40, duration: 0.8 }));
</script>
```

Rules:

1. `---` on its own line splits scenes. The first YAML block is the headmatter (project
   config). A YAML block right after a separator is that scene's frontmatter.
2. A scene body is markdown (CommonMark plus tables, via `pulldown-cmark`) with raw HTML
   allowed. `layout: raw` skips the markdown wrapper: the body is an HTML composition.
3. A trailing HTML comment is the presenter note.
4. Time model. Diapo counts steps: `steps: n` and `{step=k}` reveal markers. Clap counts
   seconds: `duration`, and `data-start` / `data-duration` / `data-track-index` on elements,
   the same attributes HyperFrames uses (https://github.com/heygen-com/hyperframes). A scene
   with steps and no duration gets `step_duration` (default 4 s) per step when rendered to
   video, which is how a deck becomes an MP4.
5. Every compiled scene exposes one contract to its host:
   `window.__scene = { duration, steps, seek(t), step(k) }`. Preview, present mode, PDF
   capture and video render all drive that contract and nothing else
   (https://www.heygen.com/research/html-to-video).
6. Determinism rules for authors: no `Date.now()`, no unseeded `Math.random()`, no network
   fetch. The compiler rejects remote `src`/`href` except fonts it can inline.
7. Assets are referenced by relative path. The web app stores them per project; the CLI reads
   them from the folder.

The format lives in one crate, `crates/scene` in this repo, package name `facile-scene`.
Clap depends on it by git tag. Features: default (parse, validate, compile to HTML), `capture`
(drive `chrome-headless-shell` over CDP: screenshot a scene at a time or step, print PDF),
`pptx` (write editable OOXML from the markdown tree, image fallback for `raw` scenes).

## Approach

Fork the shape of Docs, not its code: one Rust API in `apps/api`, one SvelteKit client in
`apps/client`, one container, one hostname. Add a Cargo workspace so `crates/scene`,
`apps/api` and `apps/cli` build together. A deck is a row holding the source text and a
revision number; every write names the revision it read and is refused when the deck moved
(`409 conflict`), which is the one rule the web editor, MCP, Bulle and `diapo push` share.
No Yjs in v1. Open editors learn of outside writes over SSE and reload the source.

User-authored HTML and JS run only inside a sandboxed frame (`sandbox="allow-scripts"`, no
`allow-same-origin`, served from `/api/decks/{id}/frame` with a `Content-Security-Policy:
sandbox` header and no cookies needed). They never run in the app origin.

## Repo layout (all new)

```
Cargo.toml                      workspace: crates/scene, apps/api, apps/cli
crates/scene/                   facile-scene: parse, compile, runtime.js, capture, pptx
apps/api/                       axum + sqlx, modules/{decks,assets,exports,share,assistant,events}, mcp/
apps/api/migrations/            0001_porte.sql, 0002_diapo.sql, 0003_mcp_actions.sql
apps/cli/                       binary `diapo`
apps/client/                    SvelteKit 2, Svelte 5, Tailwind 4, @facile/muse
scripts/check.sh                the gate, same stages as Docs
Dockerfile, docker-compose.yml, mise.toml, filet.yml, lefthook.yml, CLAUDE.md, README.md
docs/                           format.md, api.md, deployment.md, development.md
```

## API surface

All under `/api`, cookie or bearer through porte, CSRF header on cookie writes.

| Route | Does |
|---|---|
| `GET /decks`, `POST /decks` | list in the caller's spaces, create |
| `GET /decks/{id}` | metadata, source, `rev` |
| `PUT /decks/{id}/source` | body `{source, rev}`; `409` when `rev` is stale |
| `PATCH /decks/{id}` | title, space, trash |
| `GET /decks/{id}/events` | SSE: `changed {rev}` |
| `GET /decks/{id}/frame?scene=n` | compiled HTML, sandboxed |
| `POST/GET/DELETE /decks/{id}/assets` | multipart upload (sniffed, capped), list, delete |
| `POST /decks/{id}/exports` | body `{format: html|pdf|pptx|mp4}`; returns a job |
| `GET /exports/{id}` | job status, download URL |
| `POST /decks/{id}/share`, `GET /p/{token}` | public present link, read-only |
| `GET /assistant` | Bulle assistant name, as Docs does |

MCP at `/mcp` (free, scopes `diapo.read`, `diapo.write`), curated by hand:
`diapo_list_decks`, `diapo_get_deck`, `diapo_create_deck`, `diapo_write_deck` (full source,
needs `rev`), `diapo_edit_deck` (exact string replacement, refuses ambiguous matches),
`diapo_format_guide` (returns `docs/format.md`), `diapo_export`, `diapo_delete_deck`
(confirmation card).

## Steps

Wave A, one agent, proves the approach:

1. Scaffold the repo from the Docs shape: workspace, `apps/api` with tronc (`http`, `health`,
   `spa`, `migrate`, `journal`, `apiref`, `bulle`, `mcp`) and porte (`pg`, `oidc`,
   `directory`, `exchange`, `mcp`) at today's tags, `0001_porte.sql` verbatim from
   `porte::pg::SCHEMA`, empty SvelteKit client that signs in, CI, gate, Dockerfile, compose.
   `[auth: wiring copied from porte-rs docs/architecture.md, never hand-rolled]`
   `[migrations: append-only NNNN_name.sql, forward-only]` `[deploy]`
   Verify: `sh scripts/check.sh` exits 0; `docker build .` succeeds.
2. `crates/scene`: parser, validator and HTML compiler with `runtime.js` (the `__scene`
   contract, step reveals, a small seekable timeline helper). Golden tests on fixture decks.
   Verify: `cargo test -p facile-scene` passes; a fixture with steps compiles and
   `__scene.step(2)` reveals the second marker in a Playwright check.
3. Tag `v0.1.0` so Clap can pin `facile-scene`. Verify: `git ls-remote --tags origin v0.1.0`.

Wave B, parallel agents on disjoint directories, each in its own worktree:

4. `apps/api/src/modules/decks` + `events`: tables `decks`, `deck_revisions`; routes above;
   one `access()` function authorizes every deck query, `None` is 404.
   `[migrations]` `[auth]` Verify: `cargo test -p api` including a stale-`rev` 409 test and a
   cross-space 404 test.
5. `apps/api/src/modules/assets` + frame route: upload with sniffing and a size cap, the
   sandboxed frame with its CSP. Verify: test asserts the frame response carries
   `Content-Security-Policy` containing `sandbox` and no `Set-Cookie`.
6. `apps/api/src/mcp`: the eight tools on `tronc::mcp::App`, scope gates, confirmation for
   delete. `[contract: tool names are fixed by this plan; Bulle's PR depends on them]`
   Verify: `cargo test -p api --test mcp` with tronc's `mcptest` kit.
7. `apps/client` app shell and editor: deck list, split editor (source left, live preview
   right, scene strip below), present mode with keyboard and step navigation, presenter view
   with notes, Bulle sidebar (iframe to `${bulle}/embed/chat?app=diapo&id=…`, the Docs
   pattern), muse `UserMenu` and settings layout. `[ui: muse, no borders anywhere]`
   `[owner: one user menu, AI through Bulle]` Verify: `bun run check && bun run test`;
   Playwright e2e creates a deck, edits it, presents it.
8. `apps/cli`: `diapo new|serve|build|login|logout|pull|push|export|--version`, `--json`
   everywhere. `serve` watches the folder and live-reloads; `build` writes one offline HTML
   file. `[cli: standards/cli sections 4, 5, 6, 7, 8]`
   Verify: `cargo test -p diapo-cli`; `diapo build crates/scene/fixtures/basic` writes a file
   that opens offline.

Wave C, after B merges:

9. `capture` feature and PDF/HTML export jobs (`modules/exports`, one in-process worker, one
   job at a time). The API image moves from distroless to a pinned debian-slim with
   `chrome-headless-shell`; Chrome runs with every network request blocked except the frame
   and asset routes. `[deploy]` Verify: export test extracts one PDF page per slide.
10. `pptx` feature: markdown scenes become real text boxes, lists, tables and pictures;
    `raw` scenes become one picture with a note saying so. Verify: test unzips the file and
    finds the slide text in `ppt/slides/slide1.xml`; LibreOffice opens it headless.
11. MP4 export: call Clap's render API with a token exchange (audience `clap`).
    `[contract]` Verify: live test once Clap is deployed.
12. Share links and the public present page. `[auth: public route, read-only, token is
    random 32 bytes, stored hashed]` Verify: authorization test.
13. Landing page from the Docs `Landing.svelte` pattern: muse landing components, Magnetik,
    device-only downloads with `/downloads`, a mise-en-scene demo, pinned same-origin Vision
    tracker, pricing buttons to `https://sso.facile.studio/signup?lang=<locale>`, fr/en/de/es.
    `[ui]` Verify: no horizontal scroll at 360, 390, 768, 1024, 1440; `pricing-check` flow.
14. Docs: `README.md`, `CLAUDE.md`, `docs/*`, `CHANGELOG.md`. Verify: `filet check .`.

Wave D: release and deploy, in `PLAN-suite-wiring.md`.

## Boundaries

- Ask first: any change to the scene format after `v0.1.0` (Clap pins it); raising a filet
  limit (never); scaling past one replica.
- Never: run user HTML in the app origin; let capture Chrome reach the network; hand-roll
  auth; add a border in the UI; gate `/mcp` behind a plan; call a model directly instead of
  Bulle.

## Done when

`sh scripts/check.sh` and `filet check .` exit 0 on `main`; `https://diapo.facile.studio`
serves the landing signed out and the deck list signed in; a deck made on the web comes down
with `diapo pull`, goes back with `diapo push`, exports to PDF, HTML and PPTX; Bulle creates
and edits a deck from its own chat and from the in-app sidebar; `mcp-live-smoke` passes with
`MCP_TOOL=diapo_list_decks`.

## Risks

- Scope. Four exports, a CLI, an editor, MCP and a landing are several weeks of human work.
  The waves are ordered so each one leaves something usable; whatever does not land is
  listed in the session report, not silently dropped.
- PPTX fidelity: only markdown scenes are editable. That limit is shown in the export dialog.
- Chrome in the API image grows it and widens the attack surface. Network blocking and the
  sandbox frame are the mitigations; `/facile-review` covers both before release.
- No live co-editing in v1. Two people saving the same deck get a conflict prompt.
- `facile-scene` lives in this repo. If a third app needs it, extract it as encre was.

## Skip

- Yjs collaboration, comments, version history UI (revisions are stored, no UI yet).
- Importing PPTX or Google Slides.
- Themes beyond `facile` light and dark.
- A WYSIWYG slide editor. The source editor with live preview and Bulle is v1.
