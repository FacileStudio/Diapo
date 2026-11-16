function power(n) {
  return {
    in: (p) => p ** n,
    out: (p) => 1 - (1 - p) ** n,
    inOut: (p) => (p < 0.5 ? 2 ** (n - 1) * p ** n : 1 - (-2 * p + 2) ** n / 2)
  };
}

const OVERSHOOT = 1.70158;

const EASES = {
  power1: power(1),
  power2: power(2),
  power3: power(3),
  power4: power(4),
  sine: {
    in: (p) => 1 - Math.cos((p * Math.PI) / 2),
    out: (p) => Math.sin((p * Math.PI) / 2),
    inOut: (p) => -(Math.cos(Math.PI * p) - 1) / 2
  },
  expo: {
    in: (p) => (p === 0 ? 0 : 2 ** (10 * p - 10)),
    out: (p) => (p === 1 ? 1 : 1 - 2 ** (-10 * p)),
    inOut: (p) => (p === 0 || p === 1 ? p : p < 0.5 ? 2 ** (20 * p - 10) / 2 : (2 - 2 ** (-20 * p + 10)) / 2)
  },
  back: {
    in: (p) => (OVERSHOOT + 1) * p ** 3 - OVERSHOOT * p ** 2,
    out: (p) => 1 + (OVERSHOOT + 1) * (p - 1) ** 3 + OVERSHOOT * (p - 1) ** 2,
    inOut: (p) => (p < 0.5 ? EASES.back.in(p * 2) / 2 : (EASES.back.out(p * 2 - 1) + 1) / 2)
  }
};

function easing(name) {
  const [family, kind] = String(name || "power2.out").split(".");
  const found = EASES[family];
  return found ? found[kind || "out"] || found.out : (p) => p;
}
