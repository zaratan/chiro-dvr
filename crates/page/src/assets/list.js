"use strict";
const VerdictsList = ({ section, tracks, store, say, clock, shows, onChoose }) => {
  const judged = section.querySelector(".judged");
  const orphans = section.querySelector(".orphans");
  const video = section.dataset.video;
  const viewable = !section.classList.contains("no-viewer");
  const SEPARATOR = " · ";

  const update = () => {
    for (const track of tracks) {
      const value = store.get(track);
      track.row.hidden = !shows(track);
      const cell = track.row.querySelector(".verdict");
      if (!cell) continue;
      const chip = document.createElement("span");
      chip.className = `chip ${value.verdict || "todo"}`;
      chip.textContent = store.word(track);
      cell.replaceChildren(chip);
      if (value.species) {
        const species = document.createElement("span");
        species.className = "species-name";
        species.title = value.species;
        species.textContent = value.species;
        cell.append(" ", species);
      }
      const changed = store.isChanged(track);
      track.row.classList.toggle("changed", changed);
      if (changed) {
        const mark = document.createElement("span");
        mark.className = "changed-mark";
        mark.textContent = say("changed");
        cell.append(mark);
      }
    }
    const { done, total } = store.judged(video);
    judged.textContent = total > 0 ? SEPARATOR + say("judged", { done, total }) : "";
    const lost = store.orphansOf(video);
    orphans.hidden = lost.length === 0;
    if (lost.length === 0) return;
    const wasOpen = orphans.querySelector("details")?.open ?? false;
    const fold = document.createElement("details");
    fold.open = wasOpen;
    const heading = document.createElement("summary");
    heading.textContent = say("orphans", { count: lost.length });
    const why = document.createElement("p");
    why.textContent = say(viewable ? "orphans-why" : "orphans-why-no-viewer");
    const items = document.createElement("ul");
    for (const { row, nearest, word } of lost) {
      const item = document.createElement("li");
      item.textContent = say("orphan", {
        start: row.start || clock(row.startSeconds),
        track: row.track || "?",
        verdict: word,
      });
      if (viewable) {
        const button = document.createElement("button");
        button.type = "button";
        button.textContent = say("nearest", { track: nearest.id, start: nearest.clock });
        button.addEventListener("click", () => onChoose(nearest));
        item.append(" ", button);
      } else {
        item.append(" ", say("nearest-in-video", { track: nearest.id, start: nearest.clock }));
      }
      items.append(item);
    }
    fold.append(heading, why, items);
    orphans.replaceChildren(fold);
  };

  for (const track of tracks) {
    track.row.addEventListener("click", (event) => {
      if (event.target.closest("a")) return;
      onChoose(track);
    });
  }

  return { update };
};
