const TIMING = ["duration", "ease", "delay", "stagger"];

function position(timeline, at) {
  if (typeof at === "number") return at;
  if (at === "<") return timeline.previous;
  const shift = /^([+-])=([\d.]+)$/.exec(at || "");
  return shift ? timeline.cursor + (shift[1] === "-" ? -1 : 1) * Number(shift[2]) : timeline.cursor;
}

function elements(root, target) {
  if (typeof target === "string") return Array.from(root.querySelectorAll(target));
  return target instanceof Element ? [target] : Array.from(target || []);
}

function sample(track, t) {
  let active = null;
  for (const tween of track) if (tween.start <= t) active = tween;
  if (!active) return track[0].hold ? track[0].from : null;
  const progress = active.duration > 0 ? Math.min(1, (t - active.start) / active.duration) : 1;
  const eased = active.ease(progress);
  return { n: active.from.n + (active.to.n - active.from.n) * eased, unit: active.to.unit };
}

class Timeline {
  constructor(root) {
    Object.assign(this, { root, items: [], tracks: new Map(), cursor: 0, previous: 0, duration: 0 });
  }

  to(target, vars, at) {
    return this.add({ target, from: null, to: vars, at });
  }

  from(target, vars, at) {
    return this.add({ target, from: vars, to: null, at });
  }

  fromTo(target, from, to, at) {
    return this.add({ target, from, to, at });
  }

  set(target, vars, at) {
    return this.add({ target, from: null, to: { ...vars, duration: 0 }, at });
  }

  add(item) {
    this.items.push(item);
    return this;
  }

  place(item, tweens) {
    const timing = item.to || item.from;
    const found = elements(this.root, item.target);
    const start = Math.max(0, position(this, item.at) + (timing.delay || 0));
    const duration = timing.duration === undefined ? 0.5 : Math.max(0, timing.duration);
    const stagger = timing.stagger || 0;
    const ease = easing(timing.ease);
    found.forEach((el, i) => tweens.push({ el, item, start: start + i * stagger, duration, ease }));
    this.previous = start;
    this.cursor = start + duration + stagger * Math.max(0, found.length - 1);
    this.duration = Math.max(this.duration, this.cursor);
  }

  link({ el, item, start, duration, ease }) {
    const tracks = this.tracks.get(el) || {};
    this.tracks.set(el, tracks);
    const side = (vars, prop, fallback) => (vars && prop in vars ? measure(vars[prop], prop) : fallback);
    for (const prop of Object.keys(item.to || item.from).filter((key) => !TIMING.includes(key))) {
      const track = (tracks[prop] = tracks[prop] || []);
      const before = track.length ? track[track.length - 1].to : natural(el, prop);
      const tween = { start, duration, ease, hold: item.from !== null };
      track.push({ ...tween, from: side(item.from, prop, before), to: side(item.to, prop, before) });
    }
  }

  build() {
    const tweens = [];
    for (const item of this.items) this.place(item, tweens);
    tweens.sort((a, b) => a.start - b.start);
    for (const tween of tweens) this.link(tween);
    this.items = [];
    return this;
  }

  seek(t) {
    for (const [el, tracks] of this.tracks) {
      const values = {};
      for (const prop of Object.keys(tracks)) values[prop] = sample(tracks[prop], t);
      paint(el, values);
    }
  }
}
