"use strict";
const VerdictsOverlay = ({ section, tracks, store, say, current, shows, onChoose }) => {
  const REACH_PX = 12;
  const KEYS = { ArrowRight: 1, ArrowDown: 1, ArrowLeft: -1, ArrowUp: -1 };
  const byId = new Map(tracks.map((track) => [String(track.id), track]));
  const scenes = [...section.querySelectorAll(".pictures svg.scene")];
  const thumb = section.querySelector(".thumb");
  const drawn = new WeakMap();
  const lit = new WeakMap();

  const shapesOf = (svg) => {
    if (drawn.has(svg)) return drawn.get(svg);
    const shapes = new Map();
    const shape = (id) => {
      if (!shapes.has(id)) shapes.set(id, { id: Number(id), lines: [], polygons: [], circles: [] });
      return shapes.get(id);
    };
    const number = (element, name) => Number(element.getAttribute(name));
    const shown = (group) => !svg.closest(".thumb") || group.classList.contains("active");
    svg.querySelectorAll(".lines [data-id]").forEach((line) => {
      shape(line.dataset.id).lines.push(VerdictsHits.points(line.getAttribute("points")));
    });
    svg.querySelectorAll(".heads [data-id]").forEach((head) => {
      shape(head.dataset.id).polygons.push(VerdictsHits.points(head.getAttribute("points")));
    });
    svg.querySelectorAll(".leaders [data-id]").forEach((group) => {
      const line = group.querySelector("line");
      if (!line || !shown(group)) return;
      shape(group.dataset.id).lines.push([
        [number(line, "x1"), number(line, "y1")],
        [number(line, "x2"), number(line, "y2")],
      ]);
    });
    svg.querySelectorAll(".dots [data-id]").forEach((group) => {
      const circle = group.querySelector("circle");
      if (!circle || !shown(group)) return;
      shape(group.dataset.id).circles.push({ x: number(circle, "cx"), y: number(circle, "cy"), r: number(circle, "r") });
    });
    const found = [...shapes.values()].filter((one) => byId.has(String(one.id)));
    drawn.set(svg, found);
    return found;
  };

  const pick = (svg, event) => {
    const matrix = svg.getScreenCTM();
    if (!matrix || matrix.a === 0) return null;
    const point = new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse());
    const visible = shapesOf(svg).filter((shape) => shows(byId.get(String(shape.id))));
    const id = VerdictsHits.choose([point.x, point.y], visible, {
      reach: REACH_PX / matrix.a,
      current: current()?.id ?? null,
    });
    return id === null ? null : byId.get(String(id));
  };

  const light = (svg, id) => {
    const wanted = id === null ? null : String(id);
    if (lit.get(svg) === wanted) return;
    lit.set(svg, wanted);
    svg.classList.toggle("hovering", wanted !== null);
    svg.querySelectorAll("[data-id]").forEach((element) => {
      element.classList.toggle("hot", element.dataset.id === wanted);
    });
  };

  const focused = (svg) => {
    const dot = svg.contains(document.activeElement) ? document.activeElement.closest(".dots > [data-id]") : null;
    return dot ? dot.dataset.id : null;
  };

  const followPointer = (target, scene) => {
    target.addEventListener("pointermove", (event) => {
      const svg = scene();
      if (svg) light(svg, pick(svg, event)?.id ?? null);
    });
    target.addEventListener("pointerleave", () => {
      const svg = scene();
      if (svg) light(svg, focused(svg));
    });
    target.addEventListener("click", (event) => {
      if (event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      const svg = scene();
      const track = svg && pick(svg, event);
      if (!track) return;
      event.preventDefault();
      light(svg, null);
      if (track !== current()) onChoose(track);
    });
  };

  const dotsOf = (svg) => [...svg.querySelectorAll(".dots > [data-id]")].filter((dot) => byId.has(dot.dataset.id));

  const reachable = (svg) => dotsOf(svg).filter((dot) => shows(byId.get(dot.dataset.id)));

  const label = (dot) => dot.setAttribute("aria-label", VerdictsOverlay.named(store, say, byId.get(dot.dataset.id)));

  const settle = (dots, dot) => dots.forEach((one) => one.setAttribute("tabindex", one === dot ? "0" : "-1"));

  for (const svg of scenes) {
    if (dotsOf(svg).length === 0) continue;
    svg.setAttribute("role", "group");
    svg.querySelector("image")?.setAttribute("aria-hidden", "true");
    dotsOf(svg).forEach((dot) => {
      dot.setAttribute("role", "button");
      label(dot);
    });
    settle(dotsOf(svg), dotsOf(svg)[0]);
    const move = (dot) => {
      settle(dotsOf(svg), dot);
      dot.focus();
    };
    svg.addEventListener("keydown", (event) => {
      const dot = event.target.closest?.(".dots > [data-id]");
      if (!dot || event.metaKey || event.ctrlKey || event.altKey) return;
      const dots = reachable(svg);
      const index = Math.max(dots.indexOf(dot), 0);
      if (event.key in KEYS) {
        event.preventDefault();
        move(dots[(index + KEYS[event.key] + dots.length) % dots.length]);
      } else if (event.key === "Home" || event.key === "End") {
        event.preventDefault();
        move(event.key === "Home" ? dots[0] : dots[dots.length - 1]);
      } else if (event.key === "Enter" || event.key === " ") {
        event.preventDefault();
        onChoose(byId.get(dot.dataset.id));
      }
    });
    svg.addEventListener("focusin", () => light(svg, focused(svg)));
    svg.addEventListener("focusout", () => light(svg, null));
    followPointer(svg, () => svg);
  }

  if (thumb) followPointer(thumb, () => thumb.querySelector("svg.scene"));

  for (const lane of section.querySelectorAll(".lane.starts")) {
    const svg = lane.closest(".panel")?.querySelector("svg.scene");
    const nearest = (event) => {
      const starts = [...lane.querySelectorAll(".start[data-id]")].filter(
        (start) => !start.hidden && byId.has(start.dataset.id),
      );
      const shapes = starts.map((start) => {
        const box = start.getBoundingClientRect();
        return { id: Number(start.dataset.id), lines: [[[box.left + box.width / 2, 0]]], polygons: [], circles: [] };
      });
      const id = VerdictsHits.choose([event.clientX, 0], shapes, { reach: REACH_PX, current: current()?.id ?? null });
      return id === null ? null : byId.get(String(id));
    };
    const mark = (track) => {
      lane.classList.toggle("hovering", Boolean(track));
      lane.querySelectorAll(".start[data-id]").forEach((start) => {
        start.classList.toggle("hot", start.dataset.id === String(track?.id));
      });
    };
    lane.addEventListener("pointermove", (event) => {
      const track = nearest(event);
      mark(track);
      if (svg) light(svg, track?.id ?? null);
    });
    lane.addEventListener("pointerleave", () => {
      mark(null);
      if (svg) light(svg, focused(svg));
    });
    lane.addEventListener("click", (event) => {
      const track = nearest(event);
      if (track && track !== current()) onChoose(track);
    });
  }

  const restore = (track) => {
    const svg = scenes.find((one) => !one.closest(".panel")?.hidden);
    if (!svg) return;
    const all = dotsOf(svg);
    const shownDots = reachable(svg);
    const at = all.findIndex((one) => one.dataset.id === String(track?.id));
    const dot = [...all.slice(Math.max(at, 0)), ...all.slice(0, Math.max(at, 0))].find((one) => shownDots.includes(one));
    if (!dot) {
      section.querySelector(".pictures .filters [aria-pressed='true']")?.focus();
      return;
    }
    settle(all, dot);
    dot.focus();
  };

  const update = () => {
    const thumbScene = thumb?.querySelector("svg.scene");
    const verdictOf = (id) => {
      const track = byId.get(id);
      return track ? { kind: store.kindOf(track), shown: shows(track) } : null;
    };
    [...scenes, ...(thumbScene ? [thumbScene] : [])].forEach((svg) => VerdictsMarks.paint(svg, verdictOf));
    for (const svg of scenes) {
      dotsOf(svg).forEach(label);
      const dots = reachable(svg);
      if (dots.length > 0 && !dots.some((dot) => dot.getAttribute("tabindex") === "0")) settle(dotsOf(svg), dots[0]);
    }
    for (const start of section.querySelectorAll(".start[data-id]")) {
      const track = byId.get(start.dataset.id);
      if (!track) continue;
      VerdictsMarks.tag(start, store.kindOf(track));
      start.hidden = !shows(track);
    }
  };

  update();

  return { update, restore };
};

VerdictsOverlay.named = (store, say, track) => {
  const kind = store.kindOf(track);
  return say(kind === "todo" ? "trace-to-see" : `trace-${kind}`, {
    track: track.id,
    start: track.aloud,
    value: store.get(track).other,
  });
};

VerdictsOverlay.bare = (svg) => {
  svg.removeAttribute("role");
  svg.classList.remove("hovering");
  svg.querySelectorAll("[data-id]").forEach((shape) => {
    ["tabindex", "role", "aria-label"].forEach((name) => shape.removeAttribute(name));
    shape.classList.remove("hot");
  });
};
