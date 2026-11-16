const TRANSFORMS = { x: "px", y: "px", rotate: "deg", scale: "", scaleX: "", scaleY: "" };

const UNITLESS = ["opacity", "zIndex", "flexGrow", "flexShrink", "fontWeight", "order"];

function unitOf(prop) {
  if (prop in TRANSFORMS) return TRANSFORMS[prop];
  return UNITLESS.includes(prop) ? "" : "px";
}

function measure(value, prop) {
  const match = /^(-?[\d.]+(?:e-?\d+)?)([a-z%]*)$/i.exec(String(value).trim());
  if (!match) return { n: 0, unit: unitOf(prop) };
  return { n: Number(match[1]), unit: match[2] || unitOf(prop) };
}

function natural(el, prop) {
  if (prop in TRANSFORMS) return { n: prop.startsWith("scale") ? 1 : 0, unit: TRANSFORMS[prop] };
  return measure(getComputedStyle(el)[prop], prop);
}

function round(n) {
  return Math.round(n * 1e4) / 1e4;
}

function transform(values) {
  const css = (prop, fallback) => (values[prop] ? round(values[prop].n) + values[prop].unit : fallback);
  const factor = (prop) => (values[prop] ? values[prop].n : 1);
  const sx = round(factor("scale") * factor("scaleX"));
  const sy = round(factor("scale") * factor("scaleY"));
  return `translate(${css("x", "0px")}, ${css("y", "0px")}) rotate(${css("rotate", "0deg")}) scale(${sx}, ${sy})`;
}

function paint(el, values) {
  let moves = false;
  for (const [prop, value] of Object.entries(values)) {
    if (prop in TRANSFORMS) moves = true;
    else el.style[prop] = value === null ? "" : round(value.n) + value.unit;
  }
  if (!moves) return;
  const moved = Object.keys(TRANSFORMS).some((prop) => values[prop]);
  el.style.transform = moved ? transform(values) : "";
}
