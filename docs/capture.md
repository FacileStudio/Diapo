# Diapo — Capture

How a deck becomes a thumbnail, a PDF and the pictures of its animated slides: the
`diapo-capture` crate (`crates/capture`), the worker (`apps/worker`), and the isolated service
that runs them.

A compiled deck runs the HTML and JavaScript its author wrote. The browser that loads it never
runs in the API: it runs in a second service, `diapo-render`, that has no network and shares one
directory with the API. Clap renders video the same way; this page follows its design.

## The directory contract

`DATA_DIR/captures` in the API, `DIAPO_CAPTURES_DIR` (`/data/captures`) in the worker. One
directory per job, named by the job id (1 to 64 letters, digits, `-` and `_`).

| Path | Written by | Meaning |
|---|---|---|
| `<id>/bundle/` | API | `index.html`, the whole deck compiled by `scene::compile`, and each asset at the relative path the source writes |
| `<id>/job.json` | API | Queues the job |
| `<id>/cancel` | API | Any file with this name asks the worker to stop |
| `<id>/status.json` | worker | Where the job is. Rewritten atomically, twice a second while it runs |
| `<id>/thumbnail.jpg`, `out.pdf`, `stills/<n>.png` | worker | The output, renamed into place once whole |
| `.worker` | worker | The heartbeat: an RFC 3339 timestamp, rewritten every half second |

The API builds the whole job directory, bundle and `job.json`, in `DATA_DIR/staging/<id>`, which
the worker cannot see, then renames it to `captures/<id>` in one step. The worker never sees half
a job, and the API never writes into a tree the worker could have changed. Assets are copied,
not hard-linked: a hard link would hand the worker, which runs as the same uid, write access to
the asset store.

`job.json`:

```json
{"id": "4f0c…", "kind": "pdf", "width": 1920, "height": 1080, "steps": false, "scenes": null,
 "created_at": "2026-11-16T10:00:00Z"}
```

| `kind` | Output |
|---|---|
| `thumbnail` | `thumbnail.jpg`: scene 0 at its last step, a JPEG 1280 pixels wide (the stage width when it is narrower) |
| `stills` | `stills/<n>.png` for each scene of `scenes` (every scene for `null`), at its last step, at the stage size |
| `pdf` | `out.pdf`: one page per scene with every step revealed. With `steps: true`, one page per step |

`status.json`:

```json
{"state": "running", "done": 2, "total": 5, "error": null, "updated_at": "2026-11-16T10:00:03Z"}
```

`state` is `running`, `done`, `failed` or `cancelled`. A job is queued while it has a `job.json`
and no `status.json`. The types are `capture::job` and both sides use them, so the two cannot
drift.

## What the worker does

- It takes the queued job with the oldest `created_at`, one at a time.
- It checks `job.json` against its own limits, whatever the API allowed: each side of the stage
  16 to `DIAPO_CAPTURE_MAX_SIDE` pixels (4096), at most `DIAPO_CAPTURE_MAX_PAGES` pictures or
  pages (500), and `DIAPO_CAPTURE_TIMEOUT_SECONDS` of wall clock (300).
- A `cancel` file ends the job as `cancelled` within half a second plus one page.
- A job found `running` when no job runs was cut short by a restart: it is marked `failed`.
  Nothing is retried.
- It follows no symbolic link: a job directory, `bundle`, `job.json` and `status.json` that are
  links are refused, and a link where it must write is removed first.
- It never deletes a job directory. The API removes it once it has collected the output, and
  sweeps directories older than an hour.
- When the captures directory does not exist yet it waits and looks again every second. On a
  first deploy the service can start before the one-shot that creates the directory.
- `worker healthcheck` exits 0 when the heartbeat is less than 30 seconds old.

## Talking to Chrome

`crates/capture` speaks the DevTools Protocol itself over `--remote-debugging-pipe`: JSON
messages ended by a NUL byte on two inherited descriptors. No port is opened, so only the
process that started Chrome can drive it. One command at a time, with a 60 second deadline.

- **Ready** means `document.readyState` is `complete`, the fonts have loaded and
  `window.__scene.ready` is true. A deck that is not ready within 20 seconds fails the job.
- **A step** is drawn with `window.__scene.step(k)` and shot after two animation frames.
- **A still** is `Page.captureScreenshot`: PNG at the stage size, or JPEG scaled for a thumbnail.
- **The PDF** is `Page.printToPDF`. By default the deck's own print stylesheet lays it out: one
  page per scene, at the stage size, every step revealed, in one call. The text can be selected
  and searched. With `steps`, each step is printed on its own page in screen media and the
  pages are joined with `lopdf`.

`clap-render`, Clap's crate, is not a dependency. Its public API renders video frames and
exposes neither a still at a step nor a PDF, and its browser, pipe, guard and flags are private.
So `crates/capture` has its own small client and shares no code with it. See "What Clap would
need to export" below.

## Network isolation

The page may load `file://` URLs that resolve inside the bundle, and `data:`, `blob:` and
`about:` URLs. Four layers:

1. `Fetch.enable` pauses every request. The guard canonicalises each `file://` path, so `..`
   and a link that leaves the bundle are refused, and fails anything else with
   `BlockedByClient`.
2. `--proxy-server=http://127.0.0.1:1 --proxy-bypass-list=<-loopback>` sends every HTTP, HTTPS
   and WebSocket connection, loopback included, to a port nothing listens on.
3. `--host-resolver-rules=MAP * ~NOTFOUND` fails every name lookup.
4. The `diapo-render` service has no network at all.

## The `diapo-render` service

| Setting | Why |
|---|---|
| `network_mode: none` | No route to the database, the Docker network or the internet |
| `security_opt: seccomp=deploy/chrome-seccomp.json` | Chrome's sandbox needs it, see below |
| `security_opt: no-new-privileges:true`, `cap_drop: ALL` | Nothing in the container can gain a capability |
| `read_only: true`, `tmpfs: /tmp` | The only writable places are `/tmp` (Chrome's profile) and the captures directory |
| `user: 65532:65532` | Not root, and the same uid as the API, which writes the bundles |
| volume `diapo_data`, `subpath: captures`, at `/data/captures` | The worker sees the captures directory and nothing else of `/data`: no asset store, no export, no thumbnail of another deck |
| 2 CPUs, 2 GiB, 1024 PIDs | A capture cannot starve the host |

Chrome's own sandbox is on. Chrome builds it from user, PID and network namespaces and a
`chroot`, which Docker's default seccomp profile refuses to a container without
`CAP_SYS_ADMIN`. `deploy/chrome-seccomp.json` is the profile Clap ships: Docker's default (moby
`docker-v29.7.2`) with one rule added that allows `clone`, `unshare` and `chroot`. With the
default profile Chrome exits with `No usable sandbox!`. Never set `--no-sandbox` or
`DIAPO_CHROME_SANDBOX=off` in the compose file. CI sets it because GitHub's runners refuse user
namespaces to Chrome.

The subpath mount needs Docker Engine 26 or newer, and `captures` must exist in the volume
before the container is created: the one-shot `diapo-data-owner` creates it, and so does the API
at boot.

## What the API trusts

Nothing the worker writes. The browser in it loads HTML users wrote, so a compromised renderer
is the case the design plans for. The API reads `status.json`, the heartbeat and every output
through `captures::safe`:

- the file is opened with `O_NOFOLLOW` and `O_NONBLOCK`, so a link or a FIFO in its place fails;
- it must be a regular file within a cap: 64 KiB for a status or the heartbeat, 64 MiB for a
  still or a thumbnail, 300 MiB for a PDF;
- once open, the path the kernel resolved must be inside the captures directory, so a link in
  a parent (`stills` pointing at `/proc/self`) is refused too.

A `cancel` file is created new, only when the job directory is still a real one. Job directories
are removed with `remove_dir_all`, which removes a link and does not follow it. Tests plant links
for every output (`apps/api/tests/hostile.rs`).

## When the service is down

The API reads the heartbeat before it asks for anything.

| What | With the worker down |
|---|---|
| HTML export | Works |
| PPTX export | Works. Animated (`raw`) slides are placeholders and the job's `notice` says so |
| PDF export | `503 unavailable`, no job |
| Thumbnails | Not made (they are made for shared decks only). `/p/{token}/thumbnail.jpg` answers `404`, `og:image` and the oEmbed thumbnail fields are left out. The next save tries again |
| MP4 export | Unaffected: it goes to Clap |

## What Clap would need to export

To share one client instead of two, `clap-render` would have to make public:

- `cdp::Browser` (`launch`, `call`) and `pipe::spawn`: the pipe client.
- `guard::Guard` (`new`, `root`, `decide`): the request allowlist.
- `chrome::flags`, or its base flag list without the deterministic and begin-frame flags.
- `page::Page` (`open`, `eval`), with readiness left to the caller: Clap waits for
  `duration > 0`, Diapo for `ready`.
- `tools::chrome` without the `CLAP_` variable names, or with the names as parameters.

## Tests

- `cargo test -p diapo-capture --features chrome`: a real thumbnail, a still, both PDFs (page
  count, page size in points, extracted text), and a page whose `fetch` and image request reach
  no local listener.
- `cargo test -p worker`: jobs through `run`, the limits, cancel, and link safety.
- `apps/api/tests/capture.rs`: the API against a worker that follows the file protocol.

The tests that need Chrome print a `SKIPPED` banner and pass without it.

Back to the [documentation index](README.md).
