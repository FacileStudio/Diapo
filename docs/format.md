# The scene format

One text file describes a whole project: a deck for Diapo (`diapo.md`) or a video for Clap
(`clap.md`). The file sits in a folder, with the pictures and other files it uses beside it.
The crate `facile-scene` parses the file, checks it, and compiles it to one HTML document.

This page is the full reference. Everything here is checked by the parser: a file that breaks
a rule is refused with the line number of each problem.

## A complete example

````md
---
title: Q3 review
theme: facile
size: 1920x1080
layout: cover
---

# Q3 review

What moved, what did not, what comes next.

<!-- Thank the team first. -->

---
layout: split
---

## Revenue

- Up 12% {step=1}
- Churn down {step=2}

![Revenue by quarter](assets/chart.png)

---
layout: raw
duration: 6
---

<div class="clip" data-start="0" data-duration="6">
  <h1 id="t">Launch day</h1>
</div>
<script>
  scene.timeline((tl) => tl.from("#t", { opacity: 0, y: 40, duration: 0.8 }));
</script>
````

## File structure

1. A line that is exactly `---` splits the file into scenes. Spaces after the dashes are
   allowed, spaces before are not.
2. A `---` line inside a fenced code block (three or more backticks or tildes) does not split.
3. A frontmatter block may follow a `---` line. It opens when the very next line is a
   `key: value` line, and it runs to the next `---` line. Put a blank line after the
   separator when a scene's first line of text looks like `key: value`.
4. `---` directly followed by `---` is an empty frontmatter block, not an empty scene.
5. The headmatter is the frontmatter block that opens the file: the first line of the file is
   `---`. It holds the project keys, and it also holds the scene keys of the first scene.
   Without a leading `---` the file has no headmatter and every default applies.
6. A `---` at the end of the file adds no scene. A project needs at least one scene.
7. Because `---` always splits, a markdown heading underlined with `---` is not available.
   Write `## Title`. Write `***` for a horizontal rule.

Line endings may be `\n` or `\r\n`. A byte order mark is ignored.

## Frontmatter syntax

A frontmatter block is a flat list of `key: value` lines. It is a small subset of YAML:

- A key is lowercase letters, digits and `_`, followed by `:` and a space.
- A value is one scalar on one line. Lists, nested maps and multi-line values are refused.
- Quotes around a value are removed: `title: "Deck: one"`.
- On an unquoted value, ` #` starts a comment. A line that starts with `#` is a comment.
- Blank lines are allowed. Any other line is an error.
- An unknown key is an error. A key written twice in one block is an error.

## Project keys (headmatter only)

| Key | Value | Default |
|---|---|---|
| `title` | Text. The document title. | empty |
| `theme` | `facile`, the only theme. | `facile` |
| `scheme` | `light`, `dark` or `auto` (follows the viewer's system). | `light` |
| `size` | `WIDTHxHEIGHT` in pixels, each from 16 to 16384. | `1920x1080` |
| `step_duration` | Seconds one stop of a scene lasts when the scene has no `duration`. Above 0. | `4` |
| `lang` | A language tag, such as `en` or `fr-FR`. | `en` |

A project key in a scene's frontmatter is an error.

## Scene keys

| Key | Value | Default |
|---|---|---|
| `layout` | `default`, `cover`, `center`, `split` or `raw`. | `default` |
| `steps` | Whole number from 0 to 999: how many reveal steps the scene has. | the highest step the body uses |
| `duration` | Seconds, above 0. | see "Time" |
| `class` | Extra class names for the scene element: letters, digits, `-`, `_`, spaces. | none |

## Scene body

The body of a scene is markdown: CommonMark plus tables. Raw HTML is allowed anywhere
CommonMark allows it, `<style>` and `<script>` included.

With `layout: raw` the body is not markdown. It is an HTML composition, copied as written
into the scene element.

### Layouts

| Layout | What it does |
|---|---|
| `default` | Content from the top left, title first. |
| `cover` | A title scene: large heading at the bottom left, paragraphs muted. |
| `center` | Everything centered. |
| `split` | Text in the left half. Pictures (a paragraph holding only an image or a video), `figure`, tables and code blocks go to the right half. |
| `raw` | No wrapper and no padding. The scene element is the full stage, `position: absolute`. |

### Presenter note

When the body ends with an HTML comment that starts at the beginning of a line, the text of
that comment is the presenter note. It is removed from the body and never appears in the
compiled document. Any other comment stays in the body.

### Steps

A step reveals part of a scene. A scene with `n` steps has `n + 1` stops: stop 0 shows
nothing marked, stop `k` shows every element whose step is `k` or lower.

- In markdown, write `{step=k}` at the end of a paragraph, a heading, a list item or a quote.
  `k` goes from 1 to 999. The marker is removed and the block gets `data-step="k"`.
- A marker in a paragraph that sits directly in a list item or a quote reveals the whole
  item or quote.
- In HTML, in any layout, write the attribute yourself: `<p data-step="2">`.
- A marker inside a table cell is an error: put `data-step` on HTML instead.
- A marker inside inline code or a code block stays as text.
- When `steps` is set, a step above it is an error. When it is not set, it is the highest
  step used.

Several elements may share one step. Steps need not be contiguous.

## Time

Every scene has a length in seconds. In order:

1. `duration`, when the frontmatter sets it.
2. Otherwise the end of the last clip (`data-start` + `data-duration`), when the scene has one.
3. Otherwise `step_duration` for each stop: `(steps + 1) * step_duration`.

The stops of a scene share its length equally. With `steps: 2` and 12 seconds, stop 0 runs
from 0 to 4, stop 1 from 4 to 8, stop 2 from 8 to 12. That is how a deck becomes a video.

The project lasts the sum of its scenes.

### Clips

A clip is an element that exists only for a part of its scene. Use them in any layout.

| Attribute | Value |
|---|---|
| `data-start` | Seconds from the start of the scene, 0 or more. The element is hidden before. |
| `data-duration` | Seconds, 0 or more. The element is hidden after `data-start + data-duration`. Without it the element stays to the end. |
| `data-track-index` | Whole number. Becomes the element's `z-index`. |

A clip that ends at or after the end of its scene is still shown on the scene's last instant.

### Timelines

A scene animates through `scene.timeline`, a small seekable tween engine built into the
compiled document. Call it from a `<script>` placed in the scene, after the elements it
animates:

```html
<script>
  scene.timeline((tl) => {
    tl.from("#title", { opacity: 0, y: 40, duration: 0.8 });
    tl.to(".bar", { scaleY: 1.5, duration: 1, stagger: 0.1, ease: "back.out" }, "+=0.2");
  });
</script>
```

Selectors are resolved inside the scene that holds the script. A target may also be an
element or a list of elements.

| Method | Animates |
|---|---|
| `tl.from(target, vars, at?)` | From `vars` to the element's own value. The `from` values show before the tween starts. |
| `tl.to(target, vars, at?)` | From the value the element has at that time to `vars`. |
| `tl.fromTo(target, from, to, at?)` | From `from` to `to`. Timing keys are read from `to`. |
| `tl.set(target, vars, at?)` | Sets `vars` at one instant. |

Each method returns the timeline, so calls chain.

Timing keys in `vars`: `duration` (seconds, default `0.5`), `delay` (seconds), `stagger`
(seconds between elements) and `ease`.

Property keys in `vars`:

- `opacity`.
- Transforms: `x`, `y` (pixels by default), `rotate` (degrees), `scale`, `scaleX`, `scaleY`.
  The timeline writes the element's whole `transform`, so do not set one in CSS on the same
  element.
- Any CSS property with one numeric value, in camelCase: `width`, `fontSize`,
  `letterSpacing`, `borderRadius`. A number means pixels, except for `opacity`, `zIndex`,
  `flexGrow`, `flexShrink`, `fontWeight` and `order`. A string keeps its unit: `"50%"`.
- Colors and values with several parts cannot be tweened.

`at`, the position of a tween:

| Value | Start |
|---|---|
| omitted | When the previous tween of the timeline ends. |
| a number | That second of the scene. |
| `"<"` | When the previous tween starts. |
| `"+=0.5"`, `"-=0.5"` | That long after, or before, the previous tween ends. |

Eases: `linear` (or `none`), and `power1`, `power2`, `power3`, `power4`, `sine`, `expo`,
`back`, each with `.in`, `.out` or `.inOut`. The default is `power2.out`.

A timeline has no clock. Its state at time `t` depends only on `t`, so seeking to the same
time always draws the same frame.

## Files and URLs

A scene loads only files of its own project, named by a relative path such as
`assets/chart.png`. The compiler refuses:

- a remote URL (`https://…`, `http://…`, `//…`, or any other scheme), fonts included;
- an absolute path (`/logo.png`);
- a path that leaves the folder (`../logo.png`).

It checks every place that loads a file: markdown images, the attributes `src`, `poster` and
`srcset` on any element, `href` on `link`, `image`, `use` and `script`, `data` on `object`,
and `url(…)` and `@import` in CSS. `data:` URIs are allowed. A plain link (`[text](https://…)`
or `<a href>`) loads nothing and may point anywhere.

The host decides where the files come from when it compiles: the CLI inlines them, the web
app serves them. `srcset`, `url(…)` and `@import` paths are resolved the same way.

## Determinism

A compiled scene must draw the same frame for the same time, every time. In your own
scripts:

- no `Date`, no `performance.now()`, no timers, no `requestAnimationFrame` loops;
- no `Math.random()` without a fixed seed;
- no network: no `fetch`, no remote import.

Drive every animation from `scene.timeline`. CSS transitions and CSS animations run on the
wall clock, so a capture does not see them reliably.

## The compiled document

`compile` writes one HTML document for the project, `compile_scene` one for a single scene.
Both are self-contained: the theme and the runtime are inlined, nothing is fetched.

```html
<main class="stage" data-width="1920" data-height="1080">
  <section class="scene layout-split" data-index="1" data-steps="2" data-duration="12">
    <div class="content">…</div>
  </section>
</main>
```

The stage has the project's `size` and is scaled to fit the window. The shown scene has the
class `is-active`. A revealed step has `is-revealed`. A clip outside its time has `is-off`.

A `<style>` or a `<script>` written in a scene applies to the whole document. Keep selectors
specific: use ids, or a `class` on the scene.

The document runs author scripts. A host must show it in a sandboxed frame
(`sandbox="allow-scripts"`, no `allow-same-origin`), never in its own origin.

### Theme

The `facile` theme uses the system font stack and these custom properties, which a scene may
read or override: `--bg`, `--fg`, `--muted`, `--fill`, `--fill-strong`, `--scene-width`,
`--scene-height`. The base font size is 40 pixels on a 1920 by 1080 stage. The theme draws no
borders: surfaces are separated by fills.

Printing the document gives one page per scene at the stage size, every step revealed.

### The `window.__scene` contract

Every compiled document exposes one object. A preview, a presenter, a PDF capture and a video
render drive it and nothing else.

| Member | Meaning |
|---|---|
| `ready` | `true` once the document is loaded and the members below are set. |
| `duration` | Seconds: the scene's length, or the sum of all scenes for a project document. |
| `steps` | The last step. For a project document steps are global: each scene takes `steps + 1` of them, in order. |
| `seek(t)` | Draws time `t`, clamped to `0..duration`: the scene that holds `t`, the stop that holds `t`, clips and timelines at `t`. |
| `step(k)` | Draws stop `k`, clamped to `0..steps`, settled: timelines and clips are at the end of that stop. |
| `scenes` | For each scene: `start` (seconds), `duration`, `steps`, `first` (its first global step). |
| `state()` | `{ time, step }` as last drawn. |

On load the document draws `step(0)`.

A parent window drives a sandboxed frame with messages:

| Message to the frame | Effect |
|---|---|
| `{ type: "scene:seek", t }` | `seek(t)` |
| `{ type: "scene:step", k }` | `step(k)` |

| Message from the frame | When |
|---|---|
| `{ type: "scene:ready", duration, steps, scenes }` | Once, when ready. |
| `{ type: "scene:state", time, step, scene }` | After every draw. `scene` is the index of the shown scene. |

Inside the document, the right and down arrows, Page Down, Space and Enter go one step
forward; the left and up arrows and Page Up go one step back; Home and End go to the first
and the last step.

## Errors

`parse` answers every problem at once, each as `line N: message`, in line order.

| Problem | Example message |
|---|---|
| No scene | `the project has no scene` |
| Frontmatter not closed | `the frontmatter opened here is never closed by ---` |
| Bad frontmatter line | `a frontmatter line must be key: value` |
| Unknown or duplicate key | ``unknown key `colour` `` |
| Project key in a scene | `` `theme` belongs to the headmatter, not to a scene `` |
| Bad value | `` `size` must be WIDTHxHEIGHT in pixels, each from 16 to 16384 `` |
| Bad step marker | `a step marker must be {step=k} with k from 1 up` |
| Marker in a table | `a step marker must sit in a paragraph, heading, list item or quote; elsewhere use data-step` |
| Step above `steps` | ``step 3 is past `steps: 2` `` |
| Remote URL | ``remote URL `https://…`: a scene loads only files of its project`` |
| Bad asset path | ``asset path `../x.png` must be relative and stay inside the project folder`` |
