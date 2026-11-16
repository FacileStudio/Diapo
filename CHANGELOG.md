# Changelog

All notable changes to Diapo. Versions follow SemVer.

## [Unreleased]

## [0.1.0] - 2026-11-16

### Added

- `facile-scene` (`crates/scene`): the scene format shared with Clap. It parses one text file
  into scenes, reports every problem with its line, and compiles a self-contained HTML document
  with the `window.__scene` contract, step reveals, clips and a seekable timeline.
- The format reference, `docs/format.md`, also exported as `scene::FORMAT_GUIDE`.
- `apps/api`: the Rust API on tronc-rs and porte-rs. Sign-in through Registre, health routes,
  `/version`, the API reference at `/docs`, the SPA fallback, `healthcheck` and `migrate`.
- `apps/client`: the SvelteKit shell on muse. A sign-in screen, and an empty deck list once
  signed in.
- The gate (`scripts/check.sh`), filet, CI, the Dockerfile and the compose file.
