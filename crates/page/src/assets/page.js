"use strict";
const motion = matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth";
const texts = JSON.parse(document.getElementById("texts")?.textContent ?? "{}");
const say = (key, values = {}) =>
  Object.entries(values).reduce((text, [name, value]) => text.replaceAll(`{${name}}`, () => String(value)), texts[key] ?? key);
const pad = (value) => String(value).padStart(2, "0");
const clock = (seconds) => {
  const whole = Math.max(Math.floor(seconds), 0);
  return `${Math.floor(whole / 3600)}:${pad(Math.floor((whole % 3600) / 60))}:${pad(whole % 60)}`;
};
const storage = {
  get: (key) => {
    try {
      return localStorage.getItem(key);
    } catch {
      return null;
    }
  },
  set: (key, value) => {
    try {
      localStorage.setItem(key, value);
    } catch {
      return;
    }
  },
  remove: (key) => {
    try {
      localStorage.removeItem(key);
    } catch {
      return;
    }
  },
};
const DRAFT_KEY = `batdetect:verdicts:${location.pathname}`;
const CHAIN_KEY = "batdetect:chain";
const FILE_NAME = "verdicts.csv";
const REVOKE_MS = 10000;

const sections = [...document.querySelectorAll("details.video")];

sections.forEach((section) => {
  const pictures = section.querySelector(".pictures");
  const panels = [...pictures.querySelectorAll(".panel")];
  const tabs = [...pictures.querySelectorAll('[role="tab"]')];
  const choosePart = (number) => {
    tabs.forEach((tab, index) => {
      const chosen = index === number - 1;
      tab.setAttribute("aria-selected", String(chosen));
      tab.tabIndex = chosen ? 0 : -1;
    });
    panels.forEach((panel, index) => {
      panel.hidden = index !== number - 1;
    });
  };
  tabs.forEach((tab, index) => {
    tab.addEventListener("click", () => choosePart(index + 1));
    tab.addEventListener("keydown", (event) => {
      const targets = { ArrowRight: index + 1, ArrowLeft: index - 1, Home: 0, End: tabs.length - 1 };
      if (!(event.key in targets)) return;
      const target = (targets[event.key] + tabs.length) % tabs.length;
      choosePart(target + 1);
      tabs[target].focus();
      event.preventDefault();
    });
  });
});

document.querySelectorAll("a.previous").forEach((link) => {
  link.addEventListener("click", () => {
    const target = document.getElementById(link.hash.slice(1));
    if (target) target.open = true;
  });
});

const toggle = document.querySelector(".toggle-all");
const sync = () => {
  if (!toggle) return;
  toggle.textContent = sections.some((section) => !section.open) ? toggle.dataset.expand : toggle.dataset.collapse;
};
sections.forEach((section) => {
  section.addEventListener("toggle", () => {
    sync();
    if (!section.open) section.querySelector("video")?.pause();
  });
});
toggle?.addEventListener("click", () => {
  const open = sections.some((section) => !section.open);
  sections.forEach((section) => {
    section.open = open;
  });
});
sync();

const savebar = document.querySelector(".savebar");
if (savebar) {
  const owned = sections.map((section) => ({
    section,
    tracks: [...section.querySelectorAll("tbody tr[data-id]")].map((row) => ({
      video: section.dataset.video,
      id: Number(row.dataset.id),
      start: Number(row.dataset.start),
      x: row.dataset.x ? Number(row.dataset.x) : null,
      y: row.dataset.y ? Number(row.dataset.y) : null,
      clock: clock(Number(row.dataset.start)),
      clip: row.dataset.clip ?? "",
      zoom: row.dataset.zoom ?? "",
      part: row.dataset.part ?? "",
      color: row.querySelector(".dot")?.style.getPropertyValue("--dot") ?? "",
      row,
    })),
  }));
  const tracks = owned.flatMap((one) => one.tracks);
  const store = VerdictsStore.create({
    tracks,
    values: {
      yes: [say("value-yes"), say("value-yes-other")],
      no: [say("value-no"), say("value-no-other")],
      doubt: [say("value-doubt")],
    },
  });
  const dialect = {
    headings: [
      "column-video",
      "column-track",
      "column-start",
      "column-start-seconds",
      "column-verdict",
      "column-species",
      "column-start-x",
      "column-start-y",
    ].map((key) => say(key)),
    separator: say("separator"),
    decimal: say("decimal-mark"),
  };
  const bar = VerdictsBar({ root: savebar, store, say });
  const embedded = document.getElementById("verdicts-file");
  let fingerprint = "";
  let fileRows = [];
  if (embedded) {
    const bytes = Uint8Array.from(atob(embedded.dataset.bytes), (character) => character.charCodeAt(0));
    const read = VerdictsCsv.read(VerdictsCsv.decode(bytes), dialect.headings);
    if (!read) bar.failed(say("not-verdicts", { name: embedded.dataset.name }));
    fileRows = read ?? [];
    store.load(fileRows);
    fingerprint = VerdictsCsv.fingerprint(bytes);
    bar.readFrom({ name: embedded.dataset.name, date: embedded.dataset.date, candidates: embedded.dataset.candidates });
  } else {
    store.load([]);
  }

  const chained = {
    get: () => storage.get(CHAIN_KEY) !== "off",
    set: (on) => storage.set(CHAIN_KEY, on ? "on" : "off"),
  };
  const judges = owned.map(({ section, tracks: own }) =>
    VerdictsJudge({
      section,
      tracks: own,
      store,
      say,
      motion,
      chained,
      onOpen: (opening) => judges.forEach((judge, index) => owned[index].section !== opening && judge.close()),
    }),
  );
  const lists = owned.map(({ section, tracks: own }, index) =>
    VerdictsList({ section, tracks: own, store, say, clock, onChoose: (track) => judges[index].open(track) }),
  );
  const names = document.getElementById("species-names");
  const time = () => new Date().toLocaleTimeString(document.documentElement.lang, { hour: "2-digit", minute: "2-digit" });
  let downloadedAt = "";
  let asking = false;

  const keepDraft = () => {
    if (asking) return;
    if (store.pending() > 0) downloadedAt = "";
    if (!store.drifted()) {
      storage.remove(DRAFT_KEY);
      return;
    }
    storage.set(DRAFT_KEY, JSON.stringify({ fingerprint, at: time(), downloadedAt, changes: store.changes() }));
  };

  store.subscribe(() => {
    bar.update();
    lists.forEach((list) => list.update());
    const known = [...new Set(store.rows().map((row) => row.species).filter(Boolean))].sort();
    names.replaceChildren(...known.map((name) => Object.assign(document.createElement("option"), { value: name })));
    keepDraft();
  });

  const draft = JSON.parse(storage.get(DRAFT_KEY) ?? "null");
  if (draft?.changes) {
    const resume = () => {
      asking = false;
      store.apply(draft.changes);
      if (draft.downloadedAt) {
        downloadedAt = draft.downloadedAt;
        store.markSaved();
        bar.remembered(draft.downloadedAt);
      }
    };
    const drop = () => {
      storage.remove(DRAFT_KEY);
      store.load(fileRows);
    };
    const dropping = { label: say("draft-drop"), action: drop };
    if (draft.fingerprint === fingerprint) {
      resume();
      if (!draft.downloadedAt) {
        bar.offer(say("draft-same", { time: draft.at }), { label: say("download-now"), action: () => bar.download.click() }, dropping);
      }
    } else {
      asking = true;
      bar.offer(say("draft-other", { time: draft.at }), { label: say("draft-keep"), action: resume }, {
        label: say("draft-drop"),
        action: () => {
          asking = false;
          keepDraft();
        },
      });
    }
  }
  bar.update();
  lists.forEach((list) => list.update());

  bar.download.addEventListener("click", () => {
    const text = VerdictsCsv.write(store.rows(), dialect);
    const link = document.createElement("a");
    link.href = URL.createObjectURL(new Blob([text], { type: "text/csv;charset=utf-8" }));
    link.download = FILE_NAME;
    document.body.append(link);
    link.click();
    link.remove();
    window.setTimeout(() => URL.revokeObjectURL(link.href), REVOKE_MS);
    downloadedAt = time();
    store.markSaved();
    bar.downloaded(downloadedAt);
    keepDraft();
  });

  bar.reload.addEventListener("click", () => bar.file.click());
  bar.file.addEventListener("change", async () => {
    const [file] = bar.file.files;
    bar.file.value = "";
    if (!file) return;
    const bytes = new Uint8Array(await file.arrayBuffer());
    const rows = VerdictsCsv.read(VerdictsCsv.decode(bytes), dialect.headings);
    if (!rows) {
      bar.failed(say("not-verdicts", { name: file.name }));
      return;
    }
    if (store.pending() > 0 && !window.confirm(say("replace-unsaved"))) return;
    bar.cleared();
    const date = new Date(file.lastModified).toLocaleString(document.documentElement.lang);
    fingerprint = VerdictsCsv.fingerprint(bytes);
    fileRows = rows;
    store.load(rows);
    bar.readFile({ name: file.name, date });
  });

  window.addEventListener("beforeunload", (event) => {
    if (store.pending() === 0) return;
    event.preventDefault();
    event.returnValue = "";
  });
}
