"use strict";
const VerdictsList = ({ section, tracks, store, say, shows, onChoose }) => {
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
      const orphan = (start) => say("orphan", { start, track: row.track || "?", verdict: word });
      item.append(
        ...VerdictsTime.paired(
          orphan(row.start || VerdictsTime.clock(row.startSeconds)),
          orphan(VerdictsTime.aloud(row.startSeconds)),
        ),
      );
      const near = (key) =>
        VerdictsTime.paired(
          say(key, { track: nearest.id, start: nearest.clock }),
          say(key, { track: nearest.id, start: nearest.aloud }),
        );
      if (viewable) {
        const button = document.createElement("button");
        button.type = "button";
        button.append(...near("nearest"));
        button.addEventListener("click", () => onChoose(nearest));
        item.append(" ", button);
      } else {
        item.append(" ", ...near("nearest-in-video"));
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
