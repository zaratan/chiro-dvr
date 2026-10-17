"use strict";
const VerdictsFilter = ({ section, tracks, store, say, current, onChange }) => {
  const CHOICES = ["all", "todo", "yes", "no", "doubt", "other"];
  const words = {
    all: say("filter-all"),
    todo: say("filter-to-see"),
    yes: say("yes"),
    no: say("no"),
    doubt: say("doubt"),
    other: say("filter-other"),
  };
  const video = section.dataset.video;
  let chosen = "all";

  const bar = () => {
    const group = document.createElement("div");
    group.className = "filters";
    group.setAttribute("role", "group");
    group.setAttribute("aria-label", say("filter-group"));
    for (const choice of CHOICES) {
      const button = document.createElement("button");
      button.type = "button";
      button.dataset.filter = choice;
      const name = document.createElement("span");
      name.textContent = words[choice];
      const count = document.createElement("span");
      count.className = "filter-count";
      button.append(name, " ", count);
      button.addEventListener("click", () => {
        chosen = choice;
        update();
        onChange();
      });
      group.append(button);
    }
    return group;
  };

  const bars = [bar(), bar()];
  section.querySelector(".pictures").prepend(bars[0]);
  section.querySelector(".review").prepend(bars[1]);

  const update = () => {
    const counts = store.counts(video);
    const total = { ...counts, all: tracks.length };
    if (chosen === "other" && counts.other === 0) chosen = "all";
    for (const group of bars) {
      for (const button of group.querySelectorAll("button")) {
        button.hidden = button.dataset.filter === "other" && counts.other === 0;
        button.setAttribute("aria-pressed", String(button.dataset.filter === chosen));
        button.querySelector(".filter-count").textContent = String(total[button.dataset.filter]);
      }
    }
  };

  const shows = (track) => VerdictsFilter.passes(store.kindOf(track), chosen, track === current());

  update();
  return { update, shows };
};

VerdictsFilter.passes = (kind, chosen, judged) => judged || chosen === "all" || kind === chosen;
