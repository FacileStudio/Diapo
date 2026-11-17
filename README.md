# Diapo

[![ci](https://github.com/FacileStudio/Diapo/actions/workflows/ci.yml/badge.svg)](https://github.com/FacileStudio/Diapo/actions/workflows/ci.yml)

Slides for the Facile suite, written as one text file. People edit a deck on the web, agents
edit it over MCP, and the `diapo` CLI serves, builds and syncs it.

Diapo is being built in waves; [PLAN.md](PLAN.md) is the roadmap. This first release holds
the scene format and the empty app shell.

## What works today

- The scene format: markdown scenes split by `---`, per-scene frontmatter, presenter notes,
  `{step=k}` reveals, and `layout: raw` HTML compositions with a seekable timeline
- A parser that reports every problem with its line number
- A compiler that writes one self-contained HTML document, driven through `window.__scene`
- The API and client shell: sign-in through Registre, health routes, an empty deck list
- The `diapo` command line: `new`, `serve` with live reload, `build` to one offline file,
  `check`, and `login`, `pull`, `push` against a server ([docs/cli.md](docs/cli.md))

Not built yet: decks in the database, the editor, MCP and the exports.

## A deck

```md
---
title: Q3 review
---

# Q3 review

---

## Revenue

- Up 12% {step=1}
- Churn down {step=2}
```

```sh
diapo serve          # live preview on http://localhost:3030
diapo build          # one offline HTML file
```

Use the arrow keys to move through the deck. The full reference is [docs/format.md](docs/format.md).

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/FacileStudio/Diapo/main/install.sh | bash
```

Installs to `~/.local/bin` via [kit](https://github.com/FacileStudio/kit), the Facile package
manager. Pass `--bin-dir <dir>` to change that, `--source` to build from source, `--no-skill` to
skip AI agent skill registration.

Already have `kit`:

```sh
kit add diapo
```

The first `diapo` binaries are published with the next tagged release. Until then, run it from
a checkout: `cargo run -p diapo-cli -- <command>`.

## Stack

| Layer | Tech |
|---|---|
| Format | Rust, pulldown-cmark 0.13, a JavaScript runtime with no dependency |
| CLI | Rust, clap 4, ureq 3, tiny_http, notify |
| API | Rust 1.98, axum 0.8, sqlx 0.9, PostgreSQL 16, tronc-rs 0.5, porte-rs 0.8 |
| Client | SvelteKit 2, Svelte 5 (runes), Tailwind CSS 4, muse (`@facile/muse`) |
| Deploy | Docker Compose, one container and one replica behind Traefik |

## Local development

```sh
mise trust && mise install
mise run db-up
mise run dev
```

The API listens on <http://localhost:4000>; run the client from `apps/client` with
`bun run dev`. Before a commit, run `sh scripts/check.sh` then `filet check .`.

## Documentation

| Page | What's in it |
|---|---|
| [Format](docs/format.md) | The scene format, the timeline and the `window.__scene` contract |
| [Development](docs/development.md) | Local setup, tests, the quality gate |
| [Deployment](docs/deployment.md) | The image, the compose file, bumping a library |

## License

Diapo is licensed under the [GNU Affero General Public License v3.0 only](LICENSE).

---

Part of the [Facile Suite](https://facile.studio).
