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

Not built yet: decks in the database, the editor, MCP, the CLI and the exports.

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
cargo run -p facile-scene --example compile -- deck.md deck.html
```

Open `deck.html` and use the arrow keys. The full reference is [docs/format.md](docs/format.md).

## Stack

| Layer | Tech |
|---|---|
| Format | Rust, pulldown-cmark 0.13, a JavaScript runtime with no dependency |
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
