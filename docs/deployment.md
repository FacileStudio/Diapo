# Diapo — Deployment

How Diapo ships: one image, one container, one Traefik router, one hostname, one replica.
Nothing is deployed yet; the first deploy and the suite registration are in
`PLAN-suite-wiring.md`.

## Topology

```
diapo.facile.studio ──▶ Traefik (web → websecure) ──▶ diapo-api:4000 ──▶ diapo-db:5432
                                                               │
                                                     volume diapo_data at /data
```

`/api/*` is the API, `/docs` the API reference, everything else the SPA. There is no
`PathPrefix` and no `stripprefix`.

## Build

```sh
docker build -t diapo .
```

The root `Dockerfile` builds the client with bun and the `api` and `worker` binaries of the
workspace with `cargo build --release --locked -p api -p worker` (the version comes from
`apps/client/package.json` through `scripts/version.sh`, and the build fails when it cannot be
read). It ends in two images:

| Target | Base | Holds | Service |
|---|---|---|---|
| `api` (the default) | `gcr.io/distroless/cc-debian12:nonroot`, pinned by digest | `/api` and the client. No shell, no browser | `diapo-api` |
| `worker` | `debian:bookworm-slim`, pinned by digest | `/worker`, `chrome-headless-shell` pinned by version and checked against its SHA-256, its libraries, two font families, `tini` | `diapo-render` |

Both run as uid 65532. `docker-compose.yml` names each service's target, and runs a one-shot
`diapo-data-owner` (busybox) that creates `/data/captures` and hands `/data` to 65532 before the
others start. The API's entrypoint is `/api`, so `docker run <api image> migrate status` works.
`diapo-api` is held to 1536 MiB: an export builds its file in memory, up to 300 MiB, with the
assets it inlines beside it.

To move Chrome, change `CHROME_VERSION` and `CHROME_SHA256` together in the `Dockerfile`, then
run the capture tests and one PDF job in a container started with the service's options.

## Configuration

| Variable | Meaning |
|---|---|
| `DATABASE_URL` | Postgres. Compose defaults it to the `diapo-db` service |
| `POSTGRES_PASSWORD` | The password of the `diapo-db` role, used by both services. Set it before the first deploy: Postgres reads it only when it creates the volume |
| `PORT` | Listen port, `4000` |
| `APP_ENV` | `production` refuses to boot without `OIDC_ISSUER` and `PORTE_TOKEN_KEY` |
| `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | The Registre client |
| `OIDC_REDIRECT_URL`, `OIDC_SUCCESS_URL` | `https://<host>/api/auth/oidc/callback` and `https://<host>/` |
| `OIDC_CLI_AUDIENCE` | Lets `facile-cli` device tokens sign in |
| `PORTE_TOKEN_KEY` | Seals the stored OIDC tokens. The key of the frame's asset links is derived from it |
| `DATA_DIR` | Where assets are stored, `/data` |
| `MCP_RESOURCE` | `https://<host>/mcp`. Turns `/mcp` on; unset, the route does not exist |
| `MCP_STATE_KEY` | Signs the MCP confirmation state. Required with `MCP_RESOURCE` |
| `BULLE_URL`, `BULLE_CLIENT_ID` | The Bulle origin (default `https://bulle.facile.studio`) and its Registre client id (default `bulle`) |
| `PUBLIC_URL` | The origin share links, oEmbed and `og:` tags are built on. Default: the origin of `OIDC_SUCCESS_URL` |
| `EMBED_ANCESTORS` | The origins that may frame `/embed/{token}`, separated by spaces or commas. Default: `https://prisme.facile.studio` |
| `CLAP_URL` | Clap's origin, for MP4 exports. Compose defaults it to `https://clap.facile.studio`; set it empty to turn MP4 off |
| `DIAPO_CAPTURE_MAX_SIDE`, `DIAPO_CAPTURE_MAX_PAGES`, `DIAPO_CAPTURE_TIMEOUT_SECONDS` | The `diapo-render` worker's own limits: 4096 pixels, 500 pages, 300 seconds |
| `SOURCE_MAX_BYTES` | The largest deck source, 512 KiB |
| `ASSET_MAX_BYTES`, `DECK_ASSETS_MAX_BYTES` | The largest asset (25 MiB) and the most one deck holds (200 MiB) |
| `JOURNAL_URL`, `JOURNAL_TOKEN` | Ships logs to Journal |
| `TRUSTED_PROXIES`, `LOG_LEVEL` | tronc's chassis variables |

Change the Dokploy environment only with the `dokploy-env-safe` flow: the panel replaces the
whole block.

## The render service

`diapo-render` is the second service of the compose file, built from the same image with the
entrypoint `/worker`. It makes thumbnails, PDFs and the pictures of animated slides in
`chrome-headless-shell`, and it is locked down because that browser loads HTML users wrote: no
network, a read-only root, no capability, uid 65532, and only the `captures` subpath of the
`diapo_data` volume. [capture.md](capture.md) has the whole contract.

- `deploy/chrome-seccomp.json` must ship with the compose file: the service names it by a
  relative path. It is Docker's default profile plus the three system calls Chrome's sandbox
  needs. Without it Chrome exits with `No usable sandbox!` and every capture fails.
- The subpath mount needs Docker Engine 26 or newer, and `captures` must exist in the volume
  before the container is created. `diapo-data-owner` creates it. On the first deploy that adds
  the service, check that the one-shot ran before `diapo-render` was created; if the container
  failed to mount, deploy once more.
- A tool that adds a network to every service breaks `network_mode: none`. Leave this service
  out of it.
- It has no Traefik router and no port.

When it is down, HTML and PPTX exports still work (animated slides become placeholders), PDF
export answers `503`, and no thumbnail is made. Nothing else depends on it.

## Migrations

Migrations run at boot, behind sqlx's lock. `0001_porte.sql` is porte's schema verbatim,
`0002_diapo.sql` holds the spaces mirrored from Registre, the decks, their revisions and their
assets, `0003_mcp_actions.sql` is tronc's pending-action table verbatim, and
`0004_share_exports.sql` adds share links and export jobs, and `0005_export_progress.sql` adds
the progress columns of an export. Files are
append-only: never edit one that shipped.

## Health

`/health` and `/ready` (also under `/api`). The compose healthcheck re-execs the binary:
`["CMD", "/api", "healthcheck"]`.

## One replica

`docker-compose.yml` pins `deploy.replicas: 1`. Keep it until the plan says otherwise: the
`changed` events of `GET /api/decks/{id}/events` are broadcast inside the process, and the
directory, deletion, purge and sweep loops must run once.

## Bumping a library

`tronc-rs` and `porte-rs` come from their public GitHub repos, pinned by tag in
`apps/api/Cargo.toml`. tronc appears twice there, and porte pins its own tronc: all three must
name the same tag, or the build fails with two copies of `tronc` whose types do not match. So
tronc can only move to the tag the pinned porte uses. Change the tags, run
`cargo update -p tronc -p porte`, commit `Cargo.lock`, and run `sh scripts/check.sh`.

muse moves in `apps/client/package.json` (`github:FacileStudio/muse#<tag>`), followed by
`bun install`.

Back to the [documentation index](README.md).
