# Diapo — Command line

`diapo` writes, serves, builds and syncs decks from a terminal. It is the `diapo-cli` package
in `apps/cli` and needs no server for the local commands.

## Install

```sh
curl -fsSL https://raw.githubusercontent.com/FacileStudio/Diapo/main/install.sh | bash
```

Or `facile install diapo` when you already have [facile](https://github.com/FacileStudio/facile).

The first binaries are published with the next tagged release, and `facile` needs the catalog
entry at the end of this page. Until both exist, run the CLI from a checkout:
`cargo run -p diapo-cli -- <command>`.

## Commands

| Command | Does |
|---|---|
| `diapo new <name>` | Creates the folder `<name>` with a starter `diapo.md` and `assets/` |
| `diapo serve [dir] [-p port]` | Serves the deck on `http://localhost:3030` and reloads the browser on every change |
| `diapo build [dir] [-o file]` | Writes one offline HTML file, every asset inlined. Default: `<folder name>.html` |
| `diapo check [dir]` | Prints every problem as `file:line: message` and exits 1 when there is one |
| `diapo guide` | Prints the format reference, the text of [format.md](format.md) |
| `diapo login [server]` | Signs in and stores the token |
| `diapo logout` | Ends the session and removes the stored token |
| `diapo whoami` | Prints the signed-in account |
| `diapo list` | Lists your decks, id then title. `-q` prints ids only |
| `diapo pull [deck] [dir]` | Downloads a deck and its assets. `deck` is an id or the deck's web URL |
| `diapo push [dir]` | Uploads the folder's source and assets. `--create` makes a new deck |
| `diapo open [dir]` | Prints the deck's web URL and opens it. `--print` only prints |
| `diapo export` | Explains that PDF, PPTX and MP4 exports are not built yet |

`[dir]` is a deck folder or a `.md` file, and defaults to the current folder.

Every command takes three global flags:

| Flag | Does |
|---|---|
| `--json` | Prints one JSON document on stdout. Status lines move to stderr |
| `--no-color` | Disables color. `NO_COLOR` does the same |
| `--server <url>` | Uses this server instead of the stored one |

Exit codes: `0` success, `1` error, `2` usage error, `130` on Ctrl+C.

With `--json`, a failed command prints `{"error":{"code","message"}}` on stdout. The codes a
script can match: `conflict`, `local_changes`, `not_linked`, `not_logged_in`, `unauthorized`, `unreachable`,
`invalid`, `asset_refused`, `asset_too_large`, `no_deck`, and any `error.code` the server sends.

## A local deck

```sh
diapo new talk
diapo serve talk
diapo build talk        # writes talk.html
```

`serve` compiles the deck on each request, serves the folder's files beside it and watches the
folder. A problem in the file is printed in the terminal as `talk/diapo.md:12: message` and
shown in the browser as an overlay over the last deck that compiled. The page keeps its
current step across a reload.

`build` and `check` also report an asset the deck names but the folder lacks, with the line
that names it.

`check --json` answers `{"file","valid","errors":[{"line","code","message"}]}`. The `code` is
the error kind in snake case, such as `unknown_key` or `step_out_of_range`, or `missing_asset`.

## Signing in

`diapo login` asks the server how it signs people in (`GET /api/auth/config`), then runs one
of two flows from porte:

- **Browser redirect** (the default). The CLI listens on a free port of `127.0.0.1`, opens
  `/api/auth/oidc?flow=cli&port=<port>&cli_state=<nonce>`, and waits for the redirect. It
  takes only the callback that carries its own nonce, then trades the one-time code at
  `POST /api/auth/oidc/exchange`.
- **Device grant**, with `--device`, and by default on a Linux machine with no display. The
  CLI starts an RFC 8628 device sign-in at the identity provider with the `facile-cli` client,
  prints a link to approve from any browser, then trades the provider's access token at
  `POST /api/auth/oidc/device/exchange`. That token is never stored. The server mounts this
  route only when `OIDC_CLI_AUDIENCE` is set; `--device` says so when it is not.

`--no-browser` prints the link and opens nothing. `diapo logout` calls `POST /api/auth/logout`
and removes the token; it is not an error when there is none.

## Configuration

The CLI stores its login in `${XDG_CONFIG_HOME:-~/.config}/diapo/config.yml`, on every
platform:

```yaml
server_url: https://diapo.facile.studio
token: <session token>
```

The file is created with mode `0600` in a `0700` folder, and tightened to `0600` when it is
read with wider permissions. `login` and `logout` rewrite only their own two lines.

| Variable | Overrides |
|---|---|
| `DIAPO_TOKEN` | The stored token |
| `DIAPO_SERVER_URL` | The stored server |
| `DIAPO_ISSUER` | The identity provider of the device grant (default `https://sso.facile.studio`) |

Precedence: flag, then environment, then the config file, then `https://diapo.facile.studio`.
The stored token is sent only to the server that issued it: with another `--server` or
`DIAPO_SERVER_URL`, set `DIAPO_TOKEN` too. A server may be written with or without `/api`.

## Syncing with a server

```sh
diapo pull https://diapo.facile.studio/decks/<id>    # or: diapo pull <id> [dir]
diapo push                                           # from inside the folder
```

A pulled folder holds a `.diapo.json` that links it to its deck:

```json
{
  "deck": "<id>",
  "server": "https://diapo.facile.studio",
  "rev": 7,
  "source_sha256": "…",
  "assets": { "assets/chart.svg": "…" }
}
```

`rev`, `source_sha256` and `assets` record what both sides held at the last pull or push. Every
decision compares three states: that record, the folder, and the server.

**Push** validates the deck, then sends the source with the stored `rev`
(`PUT /api/decks/{id}/source`). When the deck changed on the server the API answers `409`:
the push stops before it writes anything, exits 1 with the code `conflict`, and names the two
ways out, `diapo pull` or `diapo push --force`. `--force` reads the current `rev` first, then
overwrites. A source the server refuses (`422`) is printed as `file:line: message`.
`push --create` makes a new personal deck from the folder (`POST /api/decks` with the source).
After the source, assets are synced by SHA-256:

| Asset | Push does |
|---|---|
| In the folder, absent or different on the server | Uploads it |
| On the server, deleted from the folder since the last sync | Deletes it on the server |
| On the server, never in the folder | Leaves it and names it |

**Pull** refuses to overwrite `diapo.md`, or an asset, that changed in the folder since the
last sync. `--force` takes the server's copy. It removes a local asset only when the server
dropped it and the local file is unchanged.

Assets are the files under `assets/`, named as the source writes them (`assets/chart.png`). A
hidden file is never synced. The server types each file by its bytes and takes only images,
video, audio and fonts, with names made of ASCII letters, digits, `.`, `_` and `-`; by default
an asset may hold 25 MiB and a deck's assets 200 MiB. A refused upload stops the push and
names the file and the server's reason. A pull also fetches an asset the server holds outside
`assets/`, and never writes a name that leaves the folder, is hidden, or is `diapo.md`.

## The API it calls

All HTTP lives in `apps/cli/src/api`. The base is `<server>/api`, with a bearer token.

| Call | Used by |
|---|---|
| `GET /auth/config`, `POST /auth/oidc/exchange`, `POST /auth/oidc/device/exchange` | `login` |
| `GET /auth/me`, `POST /auth/logout` | `whoami`, `login`, `logout` |
| `GET /decks`, `POST /decks` | `list`, `push --create` |
| `GET /decks/{id}`, `PUT /decks/{id}/source` | `pull`, `push` |
| `GET /decks/{id}/assets`, `POST /decks/{id}/assets` (multipart, parts `name` and `file`) | `pull`, `push` |
| `GET /decks/{id}/assets/{name}`, `DELETE /decks/{id}/assets/{name}` | `pull`, `push` |

The server's reference is [api.md](api.md). `apps/cli/tests/live.rs` runs `push` and `pull`
through the binary against the real API router on the test Postgres.

## Releases

A `v*` tag runs `.github/workflows/release.yml`, which publishes:

```
diapo_<version>_darwin_amd64.tar.gz
diapo_<version>_darwin_arm64.tar.gz
diapo_<version>_linux_amd64.tar.gz     static, musl
diapo_<version>_linux_arm64.tar.gz     static, musl
checksums.txt
```

Each archive holds the `diapo` binary at its root. `diapo --version` prints the version of
`apps/cli/Cargo.toml`, so that version must equal the tag: bump it, with
`apps/client/package.json`, in the tagged commit. The gate and the workflow both check it.

## The `facile` catalog entry

The block to add to `internal/manifest/tools.yml` in `FacileStudio/facile`. Every `auth` field
is transcribed from the code in `apps/cli/src/config` and `apps/cli/src/api/auth.rs`:

```yaml
  - name: diapo
    summary: Slides as one text file
    repo: FacileStudio/Diapo
    branch: main
    bin: diapo
    build: rust
    srcSubdir: apps/cli
    asset: diapo
    auth:
      kind: oidc-device
      defaultServerUrl: https://diapo.facile.studio
      apiSuffix: /api
      discoveryPath: /auth/config
      identityPath: /auth/me
      transport: bearer
      envToken: DIAPO_TOKEN
      envUrl: DIAPO_SERVER_URL
      oidcDevice:
        issuer: https://sso.facile.studio
        clientId: facile-cli
        scopes: openid profile email
        exchangePath: /auth/oidc/device/exchange
      sso:
        startPath: /auth/oidc
        portParam: port
        stateParam: cli_state
        extraParams: flow=cli
        callbackWith: code
        callbackPath: /
        exchangePath: /auth/oidc/exchange
        requireState: true
      store:
        kind: file
        path: ${xdgConfig}/diapo/config.yml
        format: yaml
        tokenField: token
        urlField: server_url
        mode: 0o600
        dirMode: 0o700
        preserve: true
```

Two notes for whoever adds it:

- `facile login` stores the URL with `apiSuffix`, as `https://diapo.facile.studio/api`. The CLI
  reads a server with or without `/api`, so both spellings work.
- `srcSubdir: apps/cli` is a workspace member. `apps/cli/.cargo/config.toml` sets the target
  folder to `apps/cli/target` for a build run from there, which is where the installer's
  source build looks for the binary.

Back to the [docs index](README.md).
