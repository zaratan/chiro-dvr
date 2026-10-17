"use strict";
const VerdictsJudge = ({ section, tracks, store, say, motion, chained, onOpen }) => {
  const LAYOUT_PASSES = 3;
  const SIDE_MIN = 384;
  const SIDE_MAX = 480;
  const WIDE = 1024;
  const RATIO = 4 / 3;
  const TOAST_MS = 2000;
  const body = section.querySelector(".body");
  const pictures = section.querySelector(".pictures");
  const root = section.querySelector(".judge");
  const head = root.querySelector(".judge-head");
  const band = root.querySelector(".band");
  const box = root.querySelector(".video-box");
  const video = root.querySelector("video");
  const toast = root.querySelector(".toast");
  const blocked = root.querySelector(".blocked");
  const status = root.querySelector(".status");
  const now = root.querySelector(".now");
  const dot = root.querySelector(".who .dot");
  const viewClip = root.querySelector(".view-clip");
  const viewZoom = root.querySelector(".view-zoom");
  const chain = root.querySelector(".chain input");
  const species = root.querySelector(".species");
  const toggles = [...root.querySelectorAll("[data-verdict]")];
  const noZoom = root.querySelector(".no-zoom");
  const thumb = section.querySelector(".thumb");
  const keys = {
    [say("key-yes").toLowerCase()]: "yes",
    [say("key-no").toLowerCase()]: "no",
    [say("key-doubt").toLowerCase()]: "doubt",
    [say("key-species").toLowerCase()]: "species",
    "?": "doubt",
  };
  let current = null;
  let zoomed = false;
  let toastTimer = 0;

  chain.checked = chained.get();
  chain.addEventListener("change", () => chained.set(chain.checked));

  const layout = () => {
    if (!current) return;
    const wide = window.innerWidth >= WIDE;
    section.classList.toggle("judging-wide", wide);
    if (!wide) {
      body.style.gridTemplateColumns = "";
      body.style.height = "";
      box.style.height = "";
      return;
    }
    body.style.gridTemplateColumns = `${body.clientWidth - SIDE_MIN}px ${SIDE_MIN}px`;
    for (let pass = 0; pass < LAYOUT_PASSES; pass += 1) {
      const bar = document.querySelector(".savebar")?.offsetHeight ?? 0;
      const available = window.innerHeight - bar;
      const height = Math.max(available - head.offsetHeight - band.offsetHeight, 0);
      const width = body.clientWidth;
      const side = Math.min(SIDE_MAX, Math.max(SIDE_MIN, width - Math.floor(height * RATIO)));
      body.style.gridTemplateColumns = `${width - side}px ${side}px`;
      body.style.height = `${available}px`;
      box.style.height = `${Math.min(height, Math.floor((width - side) / RATIO))}px`;
    }
  };

  const align = (behavior) => {
    const bar = document.querySelector(".savebar")?.offsetHeight ?? 0;
    window.scrollTo({ top: body.getBoundingClientRect().top + window.scrollY - bar, behavior });
  };

  const showBlocked = (text) => {
    blocked.textContent = text;
    blocked.hidden = false;
  };

  const refused = (error) => {
    if (error?.name === "NotAllowedError") showBlocked(say("blocked"));
  };

  const play = () => {
    blocked.hidden = true;
    const source = zoomed ? current.zoom : current.clip;
    if (!source) {
      video.removeAttribute("src");
      video.load();
      showBlocked(blocked.dataset.noClip);
      return;
    }
    video.src = source;
    video.play().catch(refused);
  };

  const announce = (text, after = "") => {
    status.textContent = after ? `${text}. ${after}` : text;
    toast.textContent = text;
    toast.hidden = false;
    window.clearTimeout(toastTimer);
    toastTimer = window.setTimeout(() => {
      toast.hidden = true;
    }, TOAST_MS);
  };

  const word = (value) => (value.verdict === "other" ? value.other : value.verdict ? say(value.verdict) : say("to-see"));

  const refresh = () => {
    if (!current) return;
    const value = store.get(current);
    toggles.forEach((toggle) => toggle.setAttribute("aria-pressed", String(toggle.dataset.verdict === value.verdict)));
    if (document.activeElement !== species) species.value = value.species;
  };

  const showThumb = (track) => {
    const panel = pictures.querySelectorAll(".panel")[Math.max(Number(track.part || 1) - 1, 0)];
    const link = panel?.querySelector("a.complete, a.summary");
    if (!link) return;
    const scene = panel.querySelector("svg.scene");
    let picture;
    if (scene) {
      picture = scene.cloneNode(true);
      picture.setAttribute("aria-hidden", "true");
      picture.querySelectorAll("[data-id]").forEach((shape) => {
        shape.classList.toggle("active", shape.dataset.id === String(track.id));
      });
    } else {
      picture = document.createElement("img");
      picture.alt = "";
      picture.src = link.getAttribute("href");
    }
    thumb.replaceChildren(picture);
    thumb.href = link.getAttribute("href");
    thumb.hidden = false;
  };

  const open = (track, { focus = true, said = "" } = {}) => {
    onOpen(section);
    current = track;
    zoomed = false;
    section.open = true;
    section.classList.add("judging");
    document.body.classList.add("judging-page");
    pictures.hidden = true;
    root.hidden = false;
    tracks.forEach((one) => one.row.classList.toggle("active", one === track));
    track.row.scrollIntoView({ block: "nearest", behavior: motion });
    now.textContent = say("track-of", { track: track.id, total: tracks.length, start: track.clock });
    dot.style.setProperty("--dot", track.color);
    dot.textContent = String(track.id);
    root.setAttribute("aria-label", say("judge-track", { track: track.id }));
    viewClip.setAttribute("aria-pressed", "true");
    viewZoom.setAttribute("aria-pressed", "false");
    viewZoom.hidden = !track.zoom;
    noZoom.hidden = Boolean(track.zoom);
    root.querySelector(".toggle").setAttribute("aria-label", say("verdict-of", { track: track.id }));
    species.setAttribute("aria-label", say("species-of", { track: track.id }));
    species.placeholder = say("species");
    species.value = store.get(track).species;
    showThumb(track);
    refresh();
    play();
    layout();
    align(motion);
    window.requestAnimationFrame(() => {
      layout();
      align("auto");
    });
    if (said) announce(said, now.textContent);
    else status.textContent = now.textContent;
    if (focus) root.focus({ preventScroll: true });
  };

  const close = () => {
    current = null;
    video.pause();
    root.hidden = true;
    pictures.hidden = false;
    thumb.hidden = true;
    tracks.forEach((one) => one.row.classList.remove("active"));
    section.classList.remove("judging", "judging-wide");
    document.body.classList.remove("judging-page");
    body.style.gridTemplateColumns = "";
    body.style.height = "";
  };

  const step = (delta, said = "") => {
    if (!current) return;
    const target = tracks[tracks.indexOf(current) + delta];
    if (target) open(target, { said });
    else if (said) announce(said);
  };

  const nextToSee = () => {
    if (!current) return;
    const target = store.nextToSee(current.video, current.id);
    if (target) open(tracks.find((one) => one.id === target.id));
  };

  const said = () => {
    const value = store.get(current);
    if (!value.verdict) return say("cleared", { track: current.id });
    return value.species
      ? say("announce-species", { track: current.id, verdict: word(value), species: value.species })
      : say("announce", { track: current.id, verdict: word(value) });
  };

  const decide = (verdict, { toggle }) => {
    if (!current) return;
    const chosen = toggle && store.get(current).verdict === verdict ? "" : verdict;
    store.set(current, { verdict: chosen });
    const text = said();
    if (!chain.checked || chosen === "" || chosen === "yes") announce(text);
    if (!chain.checked || chosen === "") return;
    if (chosen === "yes") {
      species.placeholder = say("species-then-enter");
      species.focus();
    } else {
      step(1, text);
    }
  };

  toggles.forEach((toggle) => toggle.addEventListener("click", () => decide(toggle.dataset.verdict, { toggle: true })));
  root.querySelectorAll(".step").forEach((button) =>
    button.addEventListener("click", () => step(Number(button.dataset.step))),
  );
  root.querySelector(".to-see").addEventListener("click", nextToSee);
  root.querySelector(".back").addEventListener("click", close);
  viewClip.addEventListener("click", () => {
    zoomed = false;
    viewClip.setAttribute("aria-pressed", "true");
    viewZoom.setAttribute("aria-pressed", "false");
    play();
  });
  viewZoom.addEventListener("click", () => {
    zoomed = true;
    viewClip.setAttribute("aria-pressed", "false");
    viewZoom.setAttribute("aria-pressed", "true");
    play();
  });

  species.addEventListener("input", () => {
    if (current) store.set(current, { species: species.value });
  });
  species.addEventListener("keydown", (event) => {
    if (event.key === "Escape") {
      event.preventDefault();
      root.focus({ preventScroll: true });
    } else if (event.key === "Enter") {
      event.preventDefault();
      root.focus({ preventScroll: true });
      if (chain.checked && current) step(1, said());
    }
  });

  video.addEventListener("error", () => {
    if (video.getAttribute("src")) showBlocked(blocked.dataset.unreadable);
  });
  video.addEventListener("playing", () => {
    blocked.hidden = true;
  });

  root.addEventListener("keydown", (event) => {
    if (event.metaKey || event.ctrlKey || event.altKey) return;
    if (event.target.closest("input, textarea, select")) return;
    const onSurface = event.target === root || event.target === video;
    const key = event.key.toLowerCase();
    if (keys[key] === "species") {
      species.focus();
    } else if (keys[key]) {
      decide(keys[key], { toggle: false });
    } else if (event.key === "Delete" || event.key === "Backspace") {
      if (current) {
        store.set(current, { verdict: "" });
        announce(said());
      }
    } else if (key === "z" && current?.zoom) {
      (zoomed ? viewClip : viewZoom).click();
    } else if (onSurface && event.key === "ArrowRight" && event.shiftKey) {
      nextToSee();
    } else if (onSurface && event.key === "ArrowRight") {
      step(1);
    } else if (onSurface && event.key === "ArrowLeft") {
      step(-1);
    } else if (onSurface && event.key === " ") {
      if (video.paused) video.play().catch(refused);
      else video.pause();
    } else {
      return;
    }
    event.preventDefault();
  });

  window.addEventListener("resize", () => {
    if (!current) return;
    layout();
    align("auto");
  });
  store.subscribe(refresh);

  return { open, close };
};
