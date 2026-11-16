const stage = { scenes: [], pending: [], time: 0, step: 0 };

function read() {
  let start = 0;
  let first = 0;
  stage.scenes = Array.from(document.querySelectorAll(".stage > .scene")).map((el) => {
    const steps = Number(el.dataset.steps) || 0;
    const scene = { el, start, first, steps, duration: Number(el.dataset.duration) || 1, timelines: [] };
    start += scene.duration;
    first += steps + 1;
    return scene;
  });
  for (const el of document.querySelectorAll("[data-track-index]")) el.style.zIndex = el.dataset.trackIndex;
}

function attach() {
  for (const { section, timeline } of stage.pending.splice(0)) {
    const scene = stage.scenes.find((known) => known.el === section) || stage.scenes[0];
    if (scene) scene.timelines.push(timeline.build());
  }
}

function during(el, t, total) {
  const start = Number(el.dataset.start) || 0;
  const end = el.dataset.duration === undefined ? Infinity : start + Number(el.dataset.duration);
  return t >= start && (t < end || end >= total);
}

function show(scene, local) {
  const slot = scene.duration / (scene.steps + 1);
  const step = Math.min(scene.steps, Math.floor(local / slot + 1e-9));
  for (const other of stage.scenes) other.el.classList.toggle("is-active", other === scene);
  for (const el of scene.el.querySelectorAll("[data-step]")) {
    el.classList.toggle("is-revealed", Number(el.dataset.step) <= step);
  }
  for (const el of scene.el.querySelectorAll("[data-start]")) {
    el.classList.toggle("is-off", !during(el, local, scene.duration));
  }
  for (const timeline of scene.timelines) timeline.seek(local);
  return step;
}

function apply(scene, local) {
  stage.step = scene.first + show(scene, local);
  stage.time = scene.start + local;
  notify("scene:state", { time: stage.time, step: stage.step, scene: stage.scenes.indexOf(scene) });
}

function seek(t) {
  const last = stage.scenes[stage.scenes.length - 1];
  if (!last) return;
  const time = Math.min(Math.max(Number(t) || 0, 0), last.start + last.duration);
  const scene = stage.scenes.find((known) => time < known.start + known.duration) || last;
  apply(scene, time - scene.start);
}

function step(k) {
  const last = stage.scenes[stage.scenes.length - 1];
  if (!last) return;
  const target = Math.min(Math.max(Math.round(Number(k) || 0), 0), last.first + last.steps);
  const scene = stage.scenes.filter((known) => known.first <= target).pop();
  const local = target - scene.first;
  const slot = scene.duration / (scene.steps + 1);
  apply(scene, local === scene.steps ? scene.duration : (local + 1) * slot - 0.001);
}
