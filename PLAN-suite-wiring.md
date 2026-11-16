# Suite wiring and release plan for Diapo and Clap

Written 2026-11-16. Covers everything outside the two app repos. Every step here changes
production or a shared repo, so the order matters more than the size.

## Goal

Register Diapo and Clap in the suite (Registre, Bulle, DNS, Dokploy, Journal, Sonde, the
Suite site, the `facile` installer) and release both to production.

## Authorization

The owner authorized on 2026-11-16, for this batch only: creating `FacileStudio/Diapo` and
`FacileStudio/Clap`, creating the `diapo` and `clap` DNS records, and shipping both to
production with their Bulle and MCP integration. Changes to Registre and Bulle are part of
that integration. Anything else in a shared repo is out of scope.

## Facts read today

- Docs registers as a Registre seed client with `directory_sync_uri`, `deletion_uri`,
  `backchannel_logout_uri`, `secret_env`, one redirect URI, an `mcp` block and a
  `token_exchange` to audience `bulle` (`Registre/seed.yaml`, `name: Docs`).
- A new seed entry whose `secret_env` is not forwarded in Registre's `docker-compose.yml`
  and set in its Dokploy env crash-loops the IdP and takes SSO down for the suite
  (wiki `projects/registre`, 2026-11-06). The secret goes in first.
- Bulle mounts an app through `SUITE_<APP>_MCP_URL` and `SUITE_<APP>_MCP_RESOURCE`
  (`Bulle/docker-compose.yml`), an app constant in `apps/api/internal/suite/config.go`,
  search words in `suite/words.go`, the tool list in `suite/mcp_tools.go`, model notes in
  `runner/mcp_notes.go`, and the page context of the embed chat in `runner/page_note.go`.
  Commit `7e0f03ee` (Docs and Feuilles) is the example to copy.
- Bulle's own Registre entry lists a `token_exchange` audience per app.
- DNS is Cloudflare, proxied A records to `65.108.2.107`; the token is `CF_API_TOKEN` in
  tiroir (wiki `tools/dokploy`).
- Env changes go through the `dokploy-env-safe` flow. Deploys are watched with
  `dokploy-deploy-wait`. Never call `compose.deploy` after a push.
- Suite app list: `Suite/src/lib/content.ts`, `APPS`.

## Steps

Prepare (no production effect):

1. Generate per app and store in tiroir: `<APP>_OIDC_CLIENT_SECRET`,
   `<APP>_PORTE_TOKEN_KEY`, `<APP>_MCP_STATE_KEY`, `<APP>_POSTGRES_PASSWORD`. Print names
   only. Verify: `tiroir list | grep -c -E '^(DIAPO|CLAP)_'` is 8.
2. Registre branch `feat/diapo-clap`: add `Diapo` and `Clap` clients to `seed.yaml`
   (redirect `https://<app>.facile.studio/api/auth/oidc/callback`, sync, deletions,
   back-channel logout, `mcp.resource https://<app>.facile.studio/mcp` with scopes
   `<app>.read <app>.write`, `token_exchange` to `bulle` with `bulle.read bulle.write`;
   Diapo also gets `token_exchange` to `clap` with `clap.write` for MP4 export), add both
   audiences to Bulle's `token_exchange`, and forward `REGISTRE_SEED_DIAPO_SECRET` and
   `REGISTRE_SEED_CLAP_SECRET` in `docker-compose.yml`. Open a PR; do not merge.
   `[auth]` `[contract]` Verify: Registre's gate (`sh scripts/check.sh`) and its seed
   validation test pass with both env vars set.
3. Bulle branch `feat/diapo-clap`: `AppDiapo`, `AppClap`, env pairs in compose and
   `.env.example`, words (`slides`, `deck`, `présentation`, `diapo`; `video`, `vidéo`,
   `montage`, `clap`), tool lists with the names fixed in the two app plans, notes that tell
   the model to call `<app>_format_guide` before writing and to pass `rev`, delete tools
   behind the approval card, embed chat page context for `app=diapo` and `app=clap`, and the
   two origins in the embed `frame-ancestors`. Open a PR; do not merge.
   `[owner: AI through Bulle, ai.access]` Verify: Bulle's gate passes.

Go live, in this order, once each app's `main` passes its gate and is tagged:

4. Registre secrets first: `dokploy-env-safe` on the Registre compose with
   `REGISTRE_SEED_DIAPO_SECRET` and `REGISTRE_SEED_CLAP_SECRET` (dry run, then `APPLY=1`).
   Verify: the dry run lists exactly two added keys and no dropped key.
5. Merge the Registre PR. Watch with `dokploy-deploy-wait`. Verify:
   `https://sso.facile.studio/.well-known/openid-configuration` answers 200 and the
   container is `running`, not `restarting`, two minutes after the deploy.
6. DNS: proxied A records `diapo` and `clap` to `65.108.2.107`. Verify: `dig +short`.
7. Dokploy: create composes `Diapo` and `Clap` in project Facile, environment production,
   GitHub source, branch `main`, autodeploy on push; set env with `dokploy-env-safe`
   (`OIDC_*`, `PORTE_TOKEN_KEY`, `MCP_RESOURCE`, `MCP_STATE_KEY`, `BULLE_URL`,
   `JOURNAL_URL`, `JOURNAL_TOKEN`, database password). `[deploy]`
   Verify: `deploy-verify` with `VERIFY_URLS="https://diapo.facile.studio/health
   https://clap.facile.studio/health"`, then load the real pages in a browser and sign in.
8. Bulle env: `SUITE_DIAPO_MCP_URL`, `SUITE_CLAP_MCP_URL` through `dokploy-env-safe`, then
   merge the Bulle PR. Verify: `mcp-live-smoke` for both apps, then ask Bulle to list decks.
9. Journal: per-app keys and both collector names in `COLLECT_SDK_APPS`
   (wiki `projects/journal`, 2026-11-15). Verify: one log line per event, not two.
10. Sonde: a monitor per app on `/health` and one on `/version`. Verify: `sonde` lists them.
11. Suite site: add both to `APPS` in `Suite/src/lib/content.ts` (category `productivity`, icons `solar:presentation-graph-bold-duotone` for Diapo and
    `solar:clapperboard-open-bold-duotone` for Clap; the same two glyphs go in Bulle's
    `apps/client/src/lib/suite-icons.ts` in step 3),
    register glyphs through `scripts/gen-icons.ts`. Verify: Suite gate, then the live wall.
12. `facile` installer catalog: entries for the `diapo` and `clap` CLIs with their `auth:`
    blocks, after each repo's release workflow has published archives.
    `[cli: standards/cli sections 2.5, 5, 6.8]` Verify: `facile add diapo,clap` installs
    both and `diapo --version` prints the tag.
13. Wiki: `projects/diapo.md`, `projects/clap.md` with compose ids, client ids, tiroir key
    names, monitor ids and the traps found on the way.

## Boundaries

- Ask first: anything that would restart Registre outside steps 4 and 5; any change to an
  existing Registre client other than Bulle's audience list; raising Bulle limits.
- Never: merge the Registre PR before step 4 is verified; print a secret; run
  `compose.deploy` by hand after a push; touch vitrine or Field; `pkill` by pattern.

## Done when

Both hosts serve their landing and app over HTTPS, sign-in works through Registre,
`mcp-live-smoke` passes for both, Bulle edits a deck and a video project from its own chat
and from each app's sidebar, both CLIs install through `facile`, and Sonde shows four green
monitors.

## Rollback

- App: revert the commit on `main`; autodeploy brings the last version back.
- Bulle: empty `SUITE_<APP>_MCP_URL` switches the app off without a deploy of code.
- Registre: revert the seed commit. The two env vars are harmless when unused.
- DNS and Dokploy composes can stay; an unreachable app answers Traefik's 404.

## Risks

- Step 5 is the only step that can cause a suite-wide outage. It runs alone, watched, with
  the revert ready.
- Cloudflare 403s some Dokploy API writes from Python; use curl as the flows do.
- ruche build load: two Rust release builds with thin LTO at once take 6+ cores. Deploy
  Diapo, wait, then Clap.
