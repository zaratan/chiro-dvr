"use strict";
const VerdictsMarks = (() => {
  const KINDS = ["yes", "no", "doubt", "other"];
  const SIGNED = ["yes", "no", "doubt"];
  const MARK_OFFSET = 0.9;
  const SIGN_HALF = 0.5;
  const MARK_FONT = 1;
  const TICK_ELBOW = 1 / 3;
  const TICK_RISE = 0.7;

  const element = (dot, name, attributes) => {
    const created = document.createElementNS(dot.namespaceURI, name);
    Object.entries(attributes).forEach(([key, value]) => created.setAttribute(key, String(value)));
    return created;
  };

  const strokes = (dot, kind, [x, y], half) => {
    if (kind === "yes") {
      const points = `${x - half},${y} ${x - half * TICK_ELBOW},${y + half * TICK_RISE} ${x + half},${y - half * TICK_RISE}`;
      return ["halo", "sign"].map((role) => element(dot, "polyline", { points, class: role }));
    }
    if (kind === "no") {
      const d = `M${x - half},${y - half}L${x + half},${y + half}M${x + half},${y - half}L${x - half},${y + half}`;
      return ["halo", "sign"].map((role) => element(dot, "path", { d, class: role }));
    }
    const sign = element(dot, "text", { x, y, class: "sign", "font-size": (half / SIGN_HALF) * MARK_FONT });
    sign.textContent = "?";
    return [sign];
  };

  const mark = (dot, kind) => {
    const badge = dot.querySelector(".mark");
    if (badge?.dataset.kind === kind) return;
    badge?.remove();
    if (!SIGNED.includes(kind)) return;
    const circle = dot.querySelector("circle");
    const [cx, cy, r] = ["cx", "cy", "r"].map((name) => Number(circle.getAttribute(name)));
    const half = r * SIGN_HALF;
    const width = dot.ownerSVGElement?.viewBox.baseVal.width || Infinity;
    const place = [Math.min(cx + r * MARK_OFFSET, width - half * 2), Math.max(cy - r * MARK_OFFSET, half * 2)];
    const created = element(dot, "g", { class: `mark mark-${kind}` });
    created.dataset.kind = kind;
    created.append(...strokes(dot, kind, place, half));
    dot.append(created);
  };

  const tag = (element, kind) => KINDS.forEach((one) => element.classList.toggle(`verdict-${one}`, one === kind));

  const paint = (svg, verdictOf) => {
    svg.querySelectorAll("[data-id]").forEach((element) => {
      const verdict = verdictOf(element.dataset.id);
      if (!verdict) return;
      tag(element, verdict.kind);
      element.classList.toggle("filtered", !verdict.shown);
    });
    svg.querySelectorAll(".dots > [data-id]").forEach((dot) => {
      const verdict = verdictOf(dot.dataset.id);
      if (verdict) mark(dot, verdict.kind);
    });
  };

  return { paint, tag };
})();
