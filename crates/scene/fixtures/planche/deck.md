---
planche: 1
size: 1280x720
---

````frame id="intro" x="0" y="0" w="1280" h="720" duration="12" name="Every shape" order="a0" fixed_duration="true"
```notes
Start with the **title**, then reveal the rest.
```

```audio id="music" start="0" duration="12" asset="assets/music.mp3" volume="0.4"
```

```text id="title" x="80" y="60" w="1120" h="140" z="a0" style.align="center" enter="fade"
# Every *shape* {step=9}
```

```sticky id="note" x="80" y="240" w="240" h="200" rot="0.1" z="a1" step="1" text="plain"
Buy milk
then ship
```

```rect id="box" x="360" y="240" w="240" h="120" z="a2" style.fill="#dbeafe" style.stroke="#1d4ed8" style.width="3" style.font="Georgia" style.size="28" step="2"
Plain **rich** box
```

```ellipse id="dot" x="640" y="240" w="160" h="160" z="a3" style.fill="#fecaca" opacity="0.5" text="none"
```

```diamond id="gem" x="840" y="240" w="160" h="160" z="a4" style.fill="#bbf7d0" text="none"
```

```line id="rule" x="80" y="480" z="a5" style.stroke="#6b6b6b" points="0 0 400 0"
```

```arrow id="pointer" x="520" y="480" z="a6" style.width="6" points="0 0 200 60"
```

```ink id="scribble" x="760" y="480" z="a7" points="0 0 0.5 20 30 0.7 40 10 0.6"
```

```image id="chart" x="1040" y="240" w="200" h="160" z="a8" fit="cover" asset="assets/chart.svg"
```

```video id="clip" x="80" y="560" w="320" h="120" z="a9" start="2" duration="4" track="1" asset="assets/clip.mp4" media_in="1.5" volume="0.8"
```

```html id="badge" x="440" y="560" w="320" h="120" z="aA"
<div class="badge">Live <em>now</em></div>
---
<p>A rule above</p>
```

```card id="link" x="800" y="560" w="400" h="120" z="aB" url="https://example.com" title="Example" description="A link card" site="example.com"
```
````

````frame id="outro" x="1480" y="0" w="1280" h="720" name="Thanks" order="a1"
```text id="thanks" x="80" y="300" w="1120" h="120" z="a0"
## Thank you
```
````

```sticky id="loose" x="3000" y="0" z="a0" text="plain"
not on a slide
```
