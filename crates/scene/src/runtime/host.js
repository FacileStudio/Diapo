const api = { ready: false, duration: 0, steps: 0, scenes: [], seek, step };

api.state = () => ({ time: stage.time, step: stage.step });

window.__scene = api;

window.scene = {
  timeline(build) {
    const section = document.currentScript ? document.currentScript.closest(".scene") : null;
    const timeline = new Timeline(section || document);
    build(timeline);
    stage.pending.push({ section, timeline });
    if (api.ready) attach();
    return timeline;
  }
};

function notify(type, detail) {
  if (window.parent !== window) window.parent.postMessage({ type, ...detail }, "*");
}

function fit() {
  const el = document.querySelector(".stage");
  if (!el) return;
  const scale = Math.min(window.innerWidth / Number(el.dataset.width), window.innerHeight / Number(el.dataset.height));
  el.style.transform = `translate(-50%, -50%) scale(${scale})`;
}

function start() {
  read();
  attach();
  const last = stage.scenes[stage.scenes.length - 1];
  api.duration = last ? last.start + last.duration : 0;
  api.steps = last ? last.first + last.steps : 0;
  api.scenes = stage.scenes.map(({ start, duration, steps, first }) => ({ start, duration, steps, first }));
  api.ready = true;
  fit();
  step(0);
  notify("scene:ready", { duration: api.duration, steps: api.steps, scenes: api.scenes });
}

function key(event) {
  const moves = { ArrowRight: 1, ArrowDown: 1, PageDown: 1, " ": 1, Enter: 1, ArrowLeft: -1, ArrowUp: -1, PageUp: -1 };
  if (event.key === "Home") step(0);
  else if (event.key === "End") step(api.steps);
  else if (moves[event.key]) step(stage.step + moves[event.key]);
  else return;
  event.preventDefault();
}

function message(event) {
  const data = event.data || {};
  if (data.type === "scene:seek") seek(data.t);
  if (data.type === "scene:step") step(data.k);
}

document.addEventListener("DOMContentLoaded", start);
window.addEventListener("resize", fit);
window.addEventListener("keydown", key);
window.addEventListener("message", message);
