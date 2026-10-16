"use strict";
const motion = matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth";

document.querySelectorAll("details.video").forEach((section) => {
  const pictures = section.querySelector(".pictures");
  const panels = [...pictures.querySelectorAll(".panel")];
  const tabs = [...pictures.querySelectorAll('[role="tab"]')];
  const player = section.querySelector(".player");
  const video = player.querySelector("video");
  const now = player.querySelector(".now");
  const announce = section.querySelector(".announce");
  const [previous, next] = player.querySelectorAll(".step");
  const rows = [...section.querySelectorAll("tbody tr")];
  const list = section.querySelector(".tracks");
  const clips = rows.map((row) => row.querySelector(".play")).filter(Boolean);

  const choosePart = (number) => {
    tabs.forEach((tab, index) => {
      const chosen = index === number - 1;
      tab.setAttribute("aria-selected", String(chosen));
      tab.tabIndex = chosen ? 0 : -1;
    });
    panels.forEach((panel, index) => { panel.hidden = index !== number - 1; });
  };

  const mark = (row) => {
    rows.forEach((other) => other.classList.remove("active"));
    row.classList.add("active");
    row.scrollIntoView({ block: "nearest", behavior: motion });
    if (row.dataset.part) choosePart(Number(row.dataset.part));
  };

  const show = (button) => {
    const row = button.closest("tr");
    section.querySelectorAll(".play").forEach((other) => other.removeAttribute("aria-current"));
    button.setAttribute("aria-current", "true");
    mark(row);
    const index = clips.findIndex((clip) => clip.closest("tr") === row);
    previous.disabled = index === 0;
    next.disabled = index === clips.length - 1;
    pictures.hidden = true;
    player.hidden = false;
    now.textContent = button.dataset.now;
    announce.textContent = button.dataset.now;
    video.src = button.dataset.src;
    video.play().catch(() => {});
    player.scrollIntoView({ block: "nearest", behavior: motion });
  };

  const showPicture = () => {
    video.pause();
    player.hidden = true;
    pictures.hidden = false;
  };

  section.querySelectorAll(".play").forEach((button) => {
    button.addEventListener("click", () => show(button));
  });

  section.querySelectorAll("button.dot").forEach((button) => {
    button.addEventListener("click", () => {
      showPicture();
      mark(button.closest("tr"));
      pictures.scrollIntoView({ block: "nearest", behavior: motion });
    });
  });

  tabs.forEach((tab, index) => {
    tab.addEventListener("click", () => {
      choosePart(index + 1);
      const first = rows.find((row) => row.dataset.part === String(index + 1));
      if (first) list.scrollTo({ top: first.offsetTop - list.querySelector("thead").offsetHeight, behavior: motion });
    });
    tab.addEventListener("keydown", (event) => {
      const step = { ArrowRight: 1, ArrowLeft: -1 }[event.key];
      if (!step) return;
      const target = (index + step + tabs.length) % tabs.length;
      choosePart(target + 1);
      tabs[target].focus();
      event.preventDefault();
    });
  });

  [previous, next].forEach((button) => {
    button.addEventListener("click", () => {
      const active = section.querySelector("tr.active");
      const index = clips.findIndex((clip) => clip.closest("tr") === active);
      const target = clips[index + Number(button.dataset.step)];
      if (target) show(target);
    });
  });

  section.querySelector(".back").addEventListener("click", () => {
    showPicture();
    pictures.querySelector(".panel:not([hidden]) a")?.focus({ preventScroll: true });
    pictures.scrollIntoView({ block: "nearest", behavior: motion });
  });

  video.addEventListener("play", () => {
    document.querySelectorAll("video").forEach((other) => {
      if (other !== video) other.pause();
    });
  });

  video.addEventListener("error", () => {
    const text = `${now.textContent} · ${player.dataset.unreadable}`;
    now.textContent = text;
    announce.textContent = text;
  });
});

document.querySelectorAll("a.previous").forEach((link) => {
  link.addEventListener("click", () => {
    const target = document.getElementById(link.hash.slice(1));
    if (target) target.open = true;
  });
});

const toggle = document.querySelector(".toggle-all");
const sections = [...document.querySelectorAll("details.video")];
const sync = () => {
  if (!toggle) return;
  toggle.textContent = sections.some((section) => !section.open) ? toggle.dataset.expand : toggle.dataset.collapse;
};
sections.forEach((section) => {
  section.addEventListener("toggle", () => {
    sync();
    if (!section.open) section.querySelector("video").pause();
  });
});
toggle?.addEventListener("click", () => {
  const open = sections.some((section) => !section.open);
  sections.forEach((section) => { section.open = open; });
});
sync();
