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

The root `Dockerfile` builds the client with bun, builds the `api` binary of the workspace with
`cargo build --release --locked -p api` (the version comes from `apps/client/package.json`
through `scripts/version.sh`, and the build fails when it cannot be read), and copies both into
`gcr.io/distroless/cc-debian12:nonroot`, pinned by digest. The API runs as `nonroot` (uid
65532). `docker-compose.yml` runs a one-shot `diapo-data-owner` service (busybox) that hands
`/data` to 65532 before `diapo-api` starts. The entrypoint is `/api`, so
`docker run diapo migrate status` works.

## Configuration

| Variable | Meaning |
|---|---|
| `DATABASE_URL` | Postgres. Compose defaults it to the `diapo-db` service |
| `PORT` | Listen port, `4000` |
| `APP_ENV` | `production` refuses to boot without `OIDC_ISSUER` and `PORTE_TOKEN_KEY` |
| `OIDC_ISSUER`, `OIDC_CLIENT_ID`, `OIDC_CLIENT_SECRET` | The Registre client |
| `OIDC_REDIRECT_URL`, `OIDC_SUCCESS_URL` | `https://<host>/api/auth/oidc/callback` and `https://<host>/` |
| `OIDC_CLI_AUDIENCE` | Lets `facile-cli` device tokens sign in |
| `PORTE_TOKEN_KEY` | Seals the stored OIDC tokens |
| `JOURNAL_URL`, `JOURNAL_TOKEN` | Ships logs to Journal |
| `TRUSTED_PROXIES`, `LOG_LEVEL` | tronc's chassis variables |

Change the Dokploy environment only with the `dokploy-env-safe` flow: the panel replaces the
whole block.

## Migrations

Migrations run at boot, behind sqlx's lock. `0001_porte.sql` is porte's schema verbatim.

## Health

`/health` and `/ready` (also under `/api`). The compose healthcheck re-execs the binary:
`["CMD", "/api", "healthcheck"]`.

## One replica

`docker-compose.yml` pins `deploy.replicas: 1`. Keep it until the plan says otherwise.

## Bumping a library

`tronc-rs` and `porte-rs` come from their public GitHub repos, pinned by tag in
`apps/api/Cargo.toml`. tronc appears twice there, and porte pins its own tronc: all three must
name the same tag, or the build fails with two copies of `tronc` whose types do not match. So
tronc can only move to the tag the pinned porte uses. Change the tags, run
`cargo update -p tronc -p porte`, commit `Cargo.lock`, and run `sh scripts/check.sh`.

muse moves in `apps/client/package.json` (`github:FacileStudio/muse#<tag>`), followed by
`bun install`.

Back to the [documentation index](README.md).
