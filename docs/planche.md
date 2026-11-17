# Decks as planche documents

A Diapo deck is a planche document under the `full` profile. Its source is the planche text
projection: plain text that reads like markdown and round-trips losslessly. This page is for
agents and CLI users who read or write that text. The normative reference is the planche spec,
section 2: https://github.com/FacileStudio/planche/blob/v0.3.0/spec/FORMAT.md

## The projection

- The file opens with front matter: `---`, `planche: 1`, `size: <w>x<h>`, `---`. Nothing else
  goes in it. The size is the stage in pixels, `1920x1080` by default.
- Each shape is one fenced block. The info string is the type, then `id="…"`, then the shape's
  keys as `key="value"` attributes. Escapes: `\"`, `\\` and `\n`.
- Styles are one attribute per field: `style.fill`, `style.stroke`, `style.width`,
  `style.font`, `style.size`, `style.align`, `style.valign`.
- A shape inside a frame is written inside the frame's fence, one backtick longer, without
  `parent`. Coordinates of such a shape are frame pixels.
- Text shapes (`text`, `sticky`, `rect`, `ellipse`, `diamond`) hold Facile Markdown in their
  body. `text="plain"` marks plain text, `text="none"` no text.
- An `html` shape holds its HTML verbatim in its body.
- The reader is strict: an unknown type or key, a duplicate id or a value of the wrong kind is
  refused with its line.

## Slides

- Every top-level `frame` is a slide. Slides run in `order` (a fractional index), then by id.
  Frames sit left to right at `y="0"` with a 200 px gap; their `w` and `h` are the stage size.
- The first slide's `name` is the deck title.
- Notes: a `notes` fence, first inside the frame, holds the presenter note in Facile Markdown.
- Steps: a shape with `step="k"` (1 to 999) appears at step `k` of its slide. Step 0 is shown
  from the start. A slide has as many steps as its highest `step`.
- Timing: `start` and `duration` on a shape make it a clip, shown from `start` for `duration`
  seconds. A frame with `fixed_duration="true"` lasts its `duration`; otherwise it lasts until
  its last clip ends, or 4 seconds per stop.
- Shapes outside any frame are kept in the document but not presented.

## Example

`````text
---
planche: 1
size: 1920x1080
---

````frame id="intro" x="0" y="0" w="1920" h="1080" name="Q3 review" order="a0"
```notes
Thank the team first.
```

```text id="title" x="160" y="120" w="1600" h="200" z="a0"
# Q3 review
```

```text id="point" x="160" y="400" w="1600" h="300" z="a1" step="1"
- Revenue is up **12%**
- Churn is down
```

```image id="chart" x="1100" y="400" w="660" h="500" z="a2" step="2" fit="contain" asset="assets/chart.png"
```
````

````frame id="outro" x="2120" y="0" w="1920" h="1080" name="Thanks" order="a1"
```text id="thanks" x="160" y="440" w="1600" h="200" z="a0" style.align="center"
# Thank you
```
````
`````

## Legacy markdown

A source in the older scene format (`docs/format.md`) is still accepted on write. It is
converted once, scene by scene:

- each scene becomes a frame named after its first heading, or `Slide N`;
- a plain markdown scene becomes editable `text` and `image` shapes placed as its layout drew
  them;
- a scene with steps, clips, tables, raw HTML or code, and every `raw` scene, becomes one locked
  `html` shape covering the frame, holding the compiled body so it looks exactly as before;
- the presenter note becomes the frame's notes, and a duration the defaults would not give
  becomes `fixed_duration`.

## Compiling

`scene::planche::to_source` compiles a document to scene source, which `scene::parse` and
`scene::compile` turn into the presented HTML. Each top-level frame is one `layout: raw` scene;
its notes are the presenter note and its `duration` is the scene duration when
`fixed_duration` is true. The headmatter holds `size` and the title. Each child shape, by `z`,
is one `div.pl.pl-<type>` placed at `left`, `top`, `width` and `height` in frame pixels, with
`transform: rotate(<rot>rad)`, `opacity`, `data-step` when `step` is above 0, and
`data-start` and `data-duration` when set.

| Type | Compiles to |
|---|---|
| `text` | The text: rich text as Markdown rendered by the theme, plain text escaped with line breaks. `style.fill` is the text colour |
| `sticky` | The text on `style.fill` (a yellow by default), padded, with a shadow |
| `rect` | The text centred on `style.fill` |
| `ellipse` | As `rect`, with `border-radius: 50%` |
| `diamond` | As `rect`, clipped to a diamond with `clip-path` |
| `image` | `<img>` of `asset`, `object-fit` from `fit` (default `contain`) |
| `video` | `<video muted playsinline>` of `asset`, `object-fit` from `fit` |
| `audio` | A hidden `<audio>` of `asset` |
| `html` | The HTML verbatim inside its box |
| `line`, `arrow` | Inline SVG through `points`; an arrow, or a line with a `head` other than `none`, ends in a head |
| `ink` | Inline SVG through `points`, pressure ignored |
| `card` | A link box: `title`, `description` and `site`, linking to `url` |

On every text type `style.font`, `style.size` and `style.align` set `font-family`, `font-size`
in pixels and `text-align`; on `sticky`, `rect`, `ellipse` and `diamond`, `style.stroke` and
`style.width` draw an outline. On strokes they are the colour and the width.

| Text alignment | Compiles to |
|---|---|
| `style.align` `left`, `center`, `right`, `justify` | `text-align` with that value |
| no `style.align` | left on `text` and `sticky`, centred on `rect`, `ellipse` and `diamond` |
| `style.valign` `top` | `display:flex;flex-direction:column;justify-content:flex-start` |
| `style.valign` `middle` | the same column with `justify-content:center` |
| `style.valign` `bottom` | the same column with `justify-content:flex-end` |
| no `style.valign` | top on `text` and `sticky`, middle on `rect`, `ellipse` and `diamond` |

A body line that the scene splitter would read as `---` or a code fence is indented
by four spaces. The compiled scene passes the same checks as any other: a remote URL in an
asset, an image or an `html` shape is refused. `enter`, `track`, `media_in`, `volume`, bindings
and nested frames are kept in the document and not compiled yet.
