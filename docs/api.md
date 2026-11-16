# Diapo — API

The HTTP API under `/api`, the MCP server at `/mcp`, and how the sandboxed frame works. The
machine-readable list of routes is at `/docs` (`/docs/openapi.json`); a test fails when a mounted
route is missing from it.

## Conventions

- **Auth** is porte's: the `session` cookie of a browser sign-in, or `Authorization: Bearer
  <token>` for the CLI. A cookie write (`POST`, `PUT`, `PATCH`, `DELETE`) must carry the suite's
  CSRF header (`porte::CSRF_HEADER_NAME`), or it answers `403`.
- **Errors** use the suite envelope from tronc: `{"error": {"code", "message"}}`.
- **`None` is `404`.** One function, `modules::decks::access::access`, answers what a caller may
  do with a deck. A deck the caller cannot see answers `404`, never `403`: its existence does not
  leak. `403` means the caller sees the deck and may not do this to it.

| Access | Who | May |
|---|---|---|
| `viewer` | (no role maps to it yet; share links will) | read the deck, its frame and its assets |
| `editor` | a `member` of the deck's space | also save the source, rename, manage assets |
| `manager` | an `admin` or `owner` of the space; the owner of a personal deck | also move, trash, restore, delete |

A deck with no space is personal: only its owner reaches it. An instance admin gets no bypass.

## Routes

| Route | Needs | Does |
|---|---|---|
| `GET /decks` | session | Every deck the caller reaches, newest first, without sources. `?space_id=` keeps one space (`404` when not a member). `?trashed=true` lists the trash |
| `POST /decks` | editor in the space | Body `{title?, space_id?, source?}`. `201` with the deck at `rev` 1. No `source` gives a one-slide starter. The title is the one given, else the source's `title`, else `Untitled` |
| `GET /decks/{id}` | viewer | `{id, space_id, title, source, rev, created_at, updated_at, trashed_at, access}` |
| `PUT /decks/{id}/source` | editor | Body `{source, rev}`. Saves and answers the deck at `rev + 1`. See "The rev rule" |
| `PATCH /decks/{id}` | editor; manager for `space_id` and `trashed` | Body `{title?, space_id?, trashed?}`. `space_id: null` moves the deck to the caller's personal decks; a move needs editor rights in the target space. `trashed: true` trashes, `false` restores. Never changes `rev` |
| `DELETE /decks/{id}` | manager | Deletes a trashed deck for good, with its revisions and assets. `412` when the deck is not in the trash |
| `GET /decks/{id}/events` | viewer | Server-sent events. See "Events" |
| `GET /decks/{id}/frame` | viewer | The deck compiled to one HTML document. `?scene=n` compiles scene `n` alone, from 0. See "The sandboxed frame" |
| `GET /decks/{id}/assets` | viewer | `{assets: [{name, mime, size, sha256, created_at, url}]}`, by name |
| `POST /decks/{id}/assets` | editor | Multipart: a `file` part, and an optional `name` part. `201` with the asset. The same name replaces the asset |
| `GET /decks/{id}/assets/{name}` | viewer, or a frame link | The bytes. Range requests work, so a video seeks |
| `DELETE /decks/{id}/assets/{name}` | editor | `204` |
| `POST /decks/{id}/share` | editor | Shares the deck. `201` with `{shared, created_at, url}` for a new link; `200` without `url` when the deck is already shared. `?rotate=true` revokes the live link first |
| `GET /decks/{id}/share` | editor | `{shared, created_at}` |
| `DELETE /decks/{id}/share` | editor | Revokes the link, `204`; `404` when the deck is not shared |
| `POST /decks/{id}/exports` | viewer | Body `{format, steps?}`: `html`, `pptx`, `pdf` or `mp4`. `201` with the job. See "Exports" |
| `GET /exports/formats` | session | Which formats would start now. See "Exports" |
| `GET /exports/{id}` | viewer of its deck | The job |
| `GET /exports/{id}/file` | viewer of its deck, or a signed link | The file, as a download |
| `GET /spaces` | session | The caller's spaces and roles, mirrored from Registre |
| `GET /assistant` | session | `{name, accent}` of the caller's Bulle assistant; `Bulle` on any failure |
| `GET /auth/session` | none | `{user, bulle_url}`; `user` is `null` when signed out |
| `GET /mcp/pending`, `POST /mcp/pending/{id}/confirm`, `POST /mcp/pending/{id}/dismiss` | the person's own browser session | The confirm page's API for agent actions that wait for the person |
| `POST /registre/sync`, `POST /registre/deletions` | Registre | The directory nudge and the signed account deletion. Mounted only when Registre is configured |

porte adds the sign-in routes (`/auth/oidc`, `/auth/oidc/callback`, `/auth/oidc/exchange`,
`/auth/oidc/device/exchange`, `/auth/sync-profile`, `/auth/backchannel-logout`, `/auth/logout`,
`/auth/me`, `/auth/config`).

The public routes need no session and are listed under "Share links".

## The rev rule

A deck is a row holding one source text and a revision number, `rev`. Every write names the
`rev` it read:

- The deck is still at that `rev`: the source is saved, `rev` goes up by one, the new source is
  kept in `deck_revisions` with its author and origin (`web` for a browser session, `cli` for a
  token, `mcp` for an agent), and open editors are told.
- The deck moved on: `409`, nothing changes. The body is tronc's `already_exists` envelope plus
  the `rev` the deck is at now:

  ```json
  {"error": {"code": "already_exists", "message": "the deck is at rev 5, not 3: read it again, then retry"}, "rev": 5}
  ```

The web editor, `diapo push` and the MCP tools all go through `modules::decks::write::save`, so
the rule is one piece of code. There is no merge: the client reloads and the person decides.

## Invalid sources

Every source is parsed with `scene::parse` before it is stored. A source that does not parse is
refused with `422` and changes nothing. The body is tronc's `invalid_argument` envelope plus
`errors`, one entry per problem in line order, with the 1-based line:

```json
{
  "error": {"code": "invalid_argument", "message": "line 4: unknown key `colour`\nline 7: remote URL ..."},
  "errors": [
    {"line": 4, "message": "unknown key `colour`"},
    {"line": 7, "message": "remote URL `https://x.test/a.png`: a scene loads only files of its project"}
  ]
}
```

The messages are the ones `docs/format.md` lists. A source over `SOURCE_MAX_BYTES` is `413`.

## Events

`GET /decks/{id}/events` is a `text/event-stream`. Each event is named `changed` and carries the
deck's revision:

```
event: changed
data: {"rev":2}
```

- The first event arrives at once and says where the deck is now. A client that loaded the deck
  and then subscribed compares that `rev` with its own and misses nothing.
- One event follows every saved source, whichever door wrote it, and every `PATCH`, asset upload
  and asset delete (same `rev`: reload the metadata or the preview).
- Before each event the server checks the reader still reads the deck. If not, the stream ends.
- A comment line keeps the connection alive every 15 seconds.

The broadcast lives in the process, which is one reason the app runs as one replica.

## Assets

An asset is a file of one deck, named by the relative path the source writes: the source says
`![](assets/chart.png)`, the asset is named `assets/chart.png`.

- **Name.** A relative path of at most 200 bytes. Each segment holds ASCII letters, digits, `.`,
  `_` and `-`, and never starts with a dot. So no `..`, no absolute path, no backslash, no
  scheme, and nothing a URL must escape; every such name is a path `scene::parse` accepts. A
  `name` part that breaks the rule is `400`. Without a `name` part the asset is named
  `assets/<file name>`, with every other character of the file name turned into `-`.
- **Type.** Sniffed from the bytes, never taken from the upload: PNG, JPEG, GIF, WebP, AVIF,
  SVG, MP4, WebM, MP3, WAV, Ogg, WOFF, WOFF2, TTF, OTF. Anything else, HTML and scripts
  included, is `400`.
- **Size.** `ASSET_MAX_BYTES` per asset and `DECK_ASSETS_MAX_BYTES` per deck; past either, `413`.
- **Storage.** `DATA_DIR/assets/<first two hex>/<sha256>`, once per content. A daily sweep
  removes files no row names, after an hour of grace.
- **Serving.** Every asset response carries `Content-Security-Policy: sandbox; default-src
  'none'; style-src 'unsafe-inline'` and `X-Content-Type-Options: nosniff`. An SVG opened on its
  own gets an opaque origin and runs no script.

## The sandboxed frame

A compiled deck runs the author's HTML and JavaScript. It must never run in the app's origin,
where it could read the session and call the API as the person.

`GET /decks/{id}/frame`, and its public twin for a shared deck, are the only routes that answer
user HTML with an HTML content type. The
client shows it in `<iframe sandbox="allow-scripts" src="/api/decks/{id}/frame">` and drives it
with `postMessage` (`docs/format.md`). The response carries:

```
Content-Security-Policy: sandbox allow-scripts; default-src 'none'; img-src 'self' data: blob:;
  media-src 'self' blob:; style-src 'unsafe-inline'; script-src 'unsafe-inline';
  font-src 'self' data:; base-uri 'none'; form-action 'none'; frame-ancestors 'self'
X-Frame-Options: SAMEORIGIN
X-Content-Type-Options: nosniff
Cache-Control: no-store
Referrer-Policy: no-referrer
```

- `sandbox` without `allow-same-origin` gives the document an opaque origin, even when someone
  opens the URL in a tab instead of a frame. It has no cookies, no storage and no same-origin
  access to the app.
- There is no `connect-src`, so `default-src 'none'` blocks `fetch`, XHR, WebSocket and
  EventSource from the document.
- The response never carries `Set-Cookie`: a layer on the route removes it whatever an inner
  layer set.
- `frame-ancestors 'self'` and `X-Frame-Options: SAMEORIGIN` let only Diapo's own pages frame it
  (tronc's default `DENY` would block the preview). The app's page policy allows `frame-src
  'self'` and the Bulle origin.

### How the frame loads assets

The frame's request for the document is a navigation started by the app's page, so it carries
the session cookie and the route checks access as usual. The document's own requests do not: its
origin is opaque, so the browser treats them as cross-site and withholds the `SameSite` session
cookie. The frame cannot authenticate with a cookie, and it must not hold a session token.

So the frame route signs the links. While it compiles, it resolves every asset the source names
through `scene::Assets` to:

```
/api/decks/{id}/assets/{name}?frame=<expires>.<hmac>
```

- `<hmac>` is HMAC-SHA256 over `{deck id}:{expires}`, hex. The key is derived from
  `PORTE_TOKEN_KEY` under the label `diapo/frame-assets/v1`, so links survive a restart and a
  rolling deploy. Without `PORTE_TOKEN_KEY` (local development) the key is random per process.
- A token reads the assets of **one deck**, for **ten minutes** (`modules::frame::TTL`), and
  nothing else: not the source, not another deck, no write. It is issued only to a caller who
  could read the deck at that moment.
- `GET /decks/{id}/assets/{name}` accepts a valid token instead of a session. A token that is
  expired, forged or signed for another deck answers `404`. With a token the response adds
  `Access-Control-Allow-Origin: *`, which a font loaded from an opaque origin needs; the link is
  the right to read, and the response carries no cookie.
- An asset the deck does not have keeps its path as written and fails to load in the frame.
- Only references in the source are signed (attributes, `srcset`, CSS `url()`). A script that
  builds an asset URL at run time gets an unsigned path, which does not load.
- The frame is `no-store` and the client reloads it on every `changed` event, which signs the
  links again. The short life means a person removed from a space stops reading its assets
  within ten minutes. A frame left open longer must be reloaded before it can load an asset,
  or a new range of a video, it has not loaded yet.

## Share links

A deck has at most one live share link. `POST /decks/{id}/share` creates it and answers its URL,
`<public url>/p/<token>`, **once**: the token is 32 random bytes, URL-safe, and only its SHA-256
is stored, so the server cannot show it again. A second `POST` answers `200` and the state
without a `url`. `?rotate=true` revokes the live link and answers a new one. A revoked link is
gone for good, and a trashed deck cannot be shared.

`<public url>` is `PUBLIC_URL`, else the origin of `OIDC_SUCCESS_URL`, else
`http://localhost:<PORT>`.

### Public routes

No session, no cookie (a layer removes any `Set-Cookie`), 5 requests a second per client with a
burst of 60, and 30 a second with a burst of 300 on the assets route, which a frame calls once
per picture. The thumbnail is `Cache-Control: private, max-age=60`, so a revoked link stops
showing it within a minute. Every one answers `404` for a token that is unknown, revoked, or on a trashed deck.
None answers the source, a presenter note, the deck id or the space.

| Route | Answers |
|---|---|
| `GET /api/public/decks/{token}` | `{title, scenes, steps, width, height, rev, frame, thumbnail}`. `steps` is the last global step, as `window.__scene.steps`. `frame` is the route below. `thumbnail` is a route, or `null` while no picture exists |
| `GET /api/public/decks/{token}/frame?scene=n` | The same sandboxed document as the private frame. Its assets load from the route below, so the deck id never shows. Its policy ends with `frame-ancestors 'self' <EMBED_ANCESTORS>`, because the embed page that shows it sits inside another site |
| `GET /api/public/decks/{token}/assets/{name}` | An asset of the shared deck, with `Access-Control-Allow-Origin: *`. The share token is the right to read, so these links do not expire while the link is live |
| `GET /p/{token}` | The present page: the client's `index.html`, with `og:title`, `og:type`, `og:url`, `og:site_name`, the oEmbed discovery `<link>`, and `og:image` once a thumbnail exists, added to its `<head>`. The title is HTML-escaped. A link that is not live answers the bare shell with `404` |
| `GET /p/{token}/thumbnail.jpg` | The first slide as a JPEG. `404` until one is rendered |
| `GET /embed/{token}` | The same viewer for another site to frame: `Content-Security-Policy: ... frame-ancestors 'self' https://prisme.facile.studio` and no `X-Frame-Options`. The origins come from `EMBED_ANCESTORS` |
| `GET /oembed?url=<share URL>&format=json` | oEmbed 1.0. `404` unless `url` is `<public url>/p/<token>` of a live link: a private deck URL, another host and an unknown token all look the same. `501` for `format=xml` |

The client draws `/p/{token}` and `/embed/{token}`: it reads `GET /api/public/decks/{token}`
and frames `frame` with `sandbox="allow-scripts"`. Every other page of the app keeps
`frame-ancestors 'none'` and `X-Frame-Options: DENY`.

oEmbed answers:

```json
{"version": "1.0", "type": "rich", "provider_name": "Diapo", "provider_url": "https://diapo.facile.studio",
 "title": "Q3 review", "width": 960, "height": 540,
 "html": "<iframe src=\"https://diapo.facile.studio/embed/<token>\" width=\"960\" height=\"540\" title=\"Q3 review\" frameborder=\"0\" allowfullscreen></iframe>"}
```

The size is 960 wide at the deck's stage ratio, shrunk to `maxwidth` and `maxheight`.
`thumbnail_url` (`<public url>/p/<token>/thumbnail.jpg`), `thumbnail_width` (1280) and
`thumbnail_height` appear only once the thumbnail exists.

### Thumbnails

Only a shared deck has a thumbnail: the share page, oEmbed and `og:image` are the only places
it shows. When a deck is shared, and after each save of a shared deck, the capture service makes
the thumbnail of its new revision: `DATA_DIR/thumbnails/<deck id>/<rev>.jpg`, a JPEG 1280 pixels wide at the stage ratio.
Saves within three seconds are captured once, one job per deck is in flight, and the newest
revision wins: once its file is in place the older ones are removed. The routes read the file
of the deck's current `rev`: when it is there, `thumbnail.jpg` serves it and `og:image`, the
oEmbed fields and `thumbnail` appear. While the service is down, or for the seconds a capture
takes, they answer as if the deck had no picture.

## Exports

`POST /decks/{id}/exports` with `{"format": "pdf", "steps": false}` queues the export of the deck
as it is now and starts it. It waits up to two seconds: a quick job is answered finished, a
slower one as it is. Read it again with `GET /exports/{id}` until `status` is `done` or `failed`.

```json
{"id": "...", "deck_id": "...", "rev": 3, "format": "pdf", "steps": false, "status": "running",
 "done": 2, "total": 12, "error": "", "notice": "", "file_name": "Q3-review.pdf", "size": 0,
 "created_at": "...", "download_url": null}
```

- `status` is `queued`, `running`, `done` or `failed`. `done` and `total` count pages or frames;
  both are 0 while unknown. `download_url` appears with `done`.
- Jobs run in two lanes, one job at a time in each: MP4, which waits on Clap for as long as a
  render takes, and every other format. A person has one export under way at a time: asking
  again for the same deck and format answers that job; asking for anything else is `429
  rate_limited` until it is over.
- An HTML or PPTX file larger than 300 MiB is refused with `413` before it is built: an HTML
  export holds each asset once per place the source names it.
- `html`: one self-contained file. `scene::compile` with every asset the deck has inlined as a
  `data:` URI; it opens offline.
- `pptx`: an editable PowerPoint file, written by `scene::pptx` (see below). The pictures of its
  animated (`raw`) slides come from the capture service. When the service is down or slower than
  a minute, the slides are placeholders, the job is still `done`, and `notice` says so.
- `pdf`: made by the capture service ([capture.md](capture.md)). One page per slide at the stage
  size with every step revealed, and text that can be selected. `steps: true` gives one page per
  step. `503 unavailable`, and no job, when the service is down.
- `mp4`: rendered by Clap. Diapo exchanges the person's Registre grant for a token of audience
  `clap` with the scope `clap.write`, posts the bundle (`index.html` and the assets, 30 frames a
  second, the stage size) to `POST <CLAP_URL>/api/renders`, follows the render, and copies the
  video here once it is done. `done` and `total` are frames. A scene lasts its `duration`, or
  `step_duration` for each of its stops. Clap's limits apply: an even size of at most 1920 by
  1080 and 600 seconds; a refusal fails the job with Clap's reason. `503`, and no job, when
  `CLAP_URL` is empty or Registre is not configured.
- `GET /exports/{id}/file` is a download (`Content-Disposition: attachment`), under a `sandbox`
  policy. An HTML export is answered as `application/octet-stream`: user HTML is never served
  as a page of this origin. `412` until the job is done.
- A signed link, `/api/exports/{id}/file?token=<expires>.<hmac>`, works for one hour without a
  session. It is signed with the frame key under the scope `export`, so a frame token never
  opens an export. `diapo_export` answers such a link for the agent to hand to the person.
- Files live in `DATA_DIR/exports/<job id>`. Jobs and files are removed after one day, at once
  when their deck is deleted for good or the person who asked is erased, and an hourly sweep
  removes any file no job names. A job left `queued` or `running` by a restart is marked
  `failed` at boot.

`GET /exports/formats` says what an export would do now, and starts no job. A client reads it
when it opens its export dialog instead of trying each format:

```json
{"formats": [
  {"format": "html", "available": true},
  {"format": "pptx", "available": true, "notes": "raw scenes as pictures"},
  {"format": "pdf", "available": false, "reason": "the capture service is not running"},
  {"format": "mp4", "available": true}
]}
```

`pdf` is available while the capture worker's heartbeat is fresh. `mp4` is available while
`CLAP_URL` is set, Registre is configured and Clap's `/health` answers within two seconds.
`pptx` is always available; its `notes` are `raw scenes as pictures` or `raw scenes as
placeholders`, depending on the worker. One answer serves every caller for 30 seconds. It is
advice: `POST` still checks, and still answers `503`.

### What a PPTX holds

`scene::pptx::write` (crate feature `pptx`) writes Office Open XML with the `zip` crate. One
scene is one slide, at the deck's stage ratio on a 13.333 inch wide slide.

| In the scene | In the slide |
|---|---|
| The first heading | The title placeholder |
| Headings, paragraphs, quotes | Paragraphs of a text box, at the theme's sizes (a stage pixel is half a point at 1920 wide) |
| List items | Paragraphs with their nesting level, bulleted or numbered |
| Bold, italic, inline code | Bold, italic, Courier New runs |
| A code block | One Courier New paragraph per line |
| A table | A table, first row as header, fills instead of borders |
| A PNG, JPEG or GIF picture | An embedded picture, at its ratio |
| Any other picture (SVG), or a missing file | A line naming it |
| The presenter note | The notes page |
| `layout: center`, `cover`, `split` | Centered text; text set at the bottom; text left, pictures and tables right |
| `scheme: dark` | The dark colours of the `facile` theme |
| A `raw` scene | A picture of its last frame, full size, when the capture service made one; else a slide whose text box says it is animated HTML. Its notes say which |
| Step markers, timelines, raw HTML in a markdown scene | Dropped: a file shows every step |

## MCP

`/mcp` is a `tronc::mcp::App` (Streamable HTTP, stateless). It is on when `MCP_RESOURCE` is set
and is free for every account. Tokens come from Registre and are checked by porte's `mcpauth`
verifier; the RFC 9728 metadata is at `/.well-known/oauth-protected-resource/mcp`. Scopes:
`diapo.read` for the read-only tools, `diapo.write` for the others.

| Tool | Scope | Input | Answers |
|---|---|---|---|
| `diapo_list_decks` | read | `space_id?`, `trashed?`, `cursor?`, `limit?` (1 to 200, default 50) | `decks: [{id, title, space_id, rev, updated_at, url}]`, `next_cursor?`, `total` |
| `diapo_get_deck` | read | `deck_id` | `id`, `title`, `space_id`, `rev`, `source`, `truncated`, `scenes`, `assets`, `trashed`, `url` |
| `diapo_format_guide` | read | none | `guide`: `scene::FORMAT_GUIDE`, the text of `docs/format.md` |
| `diapo_create_deck` | write | `title`, `source?`, `space_id?` | `id`, `title`, `rev`, `scenes`, `url` |
| `diapo_write_deck` | write | `deck_id`, `source`, `rev` | `id`, `title`, `rev`, `scenes`, `url` |
| `diapo_edit_deck` | write | `deck_id`, `old_string`, `new_string`, `rev` | `id`, `title`, `rev`, `scenes`, `url` |
| `diapo_export` | write | `deck_id`, `format` (`html`, `pdf`, `pptx`, `mp4`), `steps?`, `job_id?` | `id`, `status`, `download_url`, `file_name`, `message`. It waits up to 30 seconds. Done: `download_url` is a signed link that works for one hour without signing in. Slower: the job is answered pending, to be read by calling again with `job_id` |
| `diapo_delete_deck` | write | `deck_id` | `status` (`pending_confirmation`, `declined`, `done`), `confirm_url?`, `message` |

- `url` is the page a person opens: `https://<host>/decks/<id>`, on the origin of `MCP_RESOURCE`.
- Writes call the same service functions as REST and emit the same `changed` event. A stale
  `rev` and an invalid source are tool errors that say what to do next; nothing is saved.
- `diapo_edit_deck` replaces one exact string. It refuses an `old_string` that is not in the
  source, and one that is there more than once.
- `diapo_delete_deck` moves the deck to the trash, where it stays 30 days. The person confirms
  first: in the client when it can ask, else on the `/pending` page from `confirm_url`, in their
  own browser session. An edit made in between closes the pending action.
- The tool names are a contract: Bulle calls them.

Back to the [documentation index](README.md).
