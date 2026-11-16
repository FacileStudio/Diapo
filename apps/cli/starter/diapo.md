---
title: {{title}}
layout: cover
---

# {{title}}

One text file, one deck. Press the right arrow to start.

<!-- Presenter note: this comment is never shown on a slide. -->

---

## How a deck is written

- A line holding only `---` starts a new slide {step=1}
- Slides are markdown: lists, tables, code, pictures {step=2}
- `{step=k}` at the end of a line reveals it on the k-th key press {step=3}

> Run `diapo guide` for the whole format.

---
layout: split
---

## Text on the left, picture on the right

A `split` layout sends pictures, tables and code to the right half.

Files live beside the deck: this chart is `assets/chart.svg`.

![Bars growing quarter after quarter](assets/chart.svg)

---
layout: raw
duration: 4
---

<style>
  #launch { position: absolute; inset: 0; display: grid; place-items: center; }
  #launch h1 { font-size: 160px; margin: 0; }
  #launch p { position: absolute; left: 0; right: 0; bottom: 160px; text-align: center; color: var(--muted); }
</style>
<div id="launch">
  <h1 id="launch-title">Animate it</h1>
  <p id="launch-note" data-start="1.5">A raw slide is plain HTML with a seekable timeline.</p>
</div>
<script>
  scene.timeline((tl) => {
    tl.from("#launch-title", { opacity: 0, y: 60, duration: 1 });
    tl.to("#launch-title", { scale: 1.15, duration: 1.5, ease: "power2.inOut" }, 2);
  });
</script>

---
layout: center
---

# Your turn

Edit `diapo.md`, and the browser follows.
