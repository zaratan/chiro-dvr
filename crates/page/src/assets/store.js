"use strict";
const VerdictsStore = (() => {
  const TOLERANCE_S = 0.2;
  const ROUNDING_S = 1e-6;
  const KINDS = ["yes", "no", "doubt"];
  const EMPTY = Object.freeze({ verdict: "", other: "", species: "" });

  const create = ({ tracks, values, say }) => {
    const videos = new Set(tracks.map((track) => track.video));
    const keyOf = (track) => `${track.video}\u0000${track.id}`;
    const byKey = new Map(tracks.map((track) => [keyOf(track), track]));
    let state = new Map();
    let loaded = new Map();
    let saved = new Map();
    let orphans = [];
    let absent = [];
    const listeners = [];

    const kind = (raw) => {
      const value = raw.trim().toLowerCase();
      if (value === "") return "";
      return KINDS.find((one) => values[one].some((word) => value === word.toLowerCase())) ?? "other";
    };

    const entry = (row) => {
      const verdict = kind(row.verdict);
      return { verdict, other: verdict === "other" ? row.verdict.trim() : "", species: row.species.trim() };
    };

    const same = (one, other) =>
      one.verdict === other.verdict && one.other === other.other && one.species === other.species;

    const attach = (rows) => {
      const attached = new Map();
      const lost = [];
      const elsewhere = [];
      const groups = new Map();
      for (const row of rows) {
        if (!videos.has(row.video) || row.startSeconds === null) {
          elsewhere.push(row);
        } else {
          if (!groups.has(row.video)) groups.set(row.video, []);
          groups.get(row.video).push(row);
        }
      }
      for (const [video, group] of groups) {
        const candidates = tracks.filter((track) => track.video === video);
        const pairs = [];
        group.forEach((row, rowIndex) => {
          candidates.forEach((track, trackIndex) => {
            const gap = Math.abs(row.startSeconds - track.start);
            if (gap > TOLERANCE_S + ROUNDING_S) return;
            const measured = row.startX !== null && track.x !== null && row.startY !== null && track.y !== null;
            const distance = measured ? Math.hypot(row.startX - track.x, row.startY - track.y) : Infinity;
            pairs.push({ rowIndex, trackIndex, gap, distance });
          });
        });
        pairs.sort((one, other) => (one.distance === other.distance ? one.gap - other.gap : one.distance < other.distance ? -1 : 1));
        const usedRows = new Set();
        const usedTracks = new Set();
        for (const pair of pairs) {
          if (usedRows.has(pair.rowIndex) || usedTracks.has(pair.trackIndex)) continue;
          usedRows.add(pair.rowIndex);
          usedTracks.add(pair.trackIndex);
          attached.set(keyOf(candidates[pair.trackIndex]), entry(group[pair.rowIndex]));
        }
        group.forEach((row, rowIndex) => {
          if (!usedRows.has(rowIndex)) lost.push(row);
        });
      }
      return { attached, lost, elsewhere };
    };

    const notify = () => listeners.forEach((listener) => listener());

    const load = (rows) => {
      const { attached, lost, elsewhere } = attach(rows);
      state = attached;
      loaded = new Map(attached);
      saved = new Map(attached);
      orphans = lost;
      absent = elsewhere;
      notify();
    };

    const changes = () =>
      tracks.filter((track) => differs(track, loaded)).map((track) => line(track, get(track)));

    const apply = (rows) => {
      const { attached, lost, elsewhere } = attach(rows);
      for (const [key, value] of attached) {
        if (value.verdict === "" && value.species === "") state.delete(key);
        else state.set(key, value);
      }
      orphans = [...orphans, ...lost];
      absent = [...absent, ...elsewhere];
      notify();
    };

    const get = (track) => state.get(keyOf(track)) ?? EMPTY;

    const set = (track, change) => {
      const next = { ...get(track), ...change };
      if (change.verdict !== undefined) next.other = "";
      if (next.verdict === "" && next.other === "" && next.species === "") {
        state.delete(keyOf(track));
      } else {
        state.set(keyOf(track), next);
      }
      notify();
    };

    const differs = (track, from) => !same(get(track), from.get(keyOf(track)) ?? EMPTY);

    const isChanged = (track) => differs(track, saved);

    const pending = () => tracks.filter(isChanged).length;

    const drifted = () => tracks.some((track) => differs(track, loaded));

    const markSaved = () => {
      saved = new Map(state);
      notify();
    };

    const judgeable = tracks.filter((track) => track.judgeable);

    const kindOf = (track) => get(track).verdict || "todo";

    const wordOf = (value) => {
      if (value.verdict === "other") return say("other", { value: value.other });
      return say(value.verdict || "to-see");
    };

    const word = (track) => wordOf(get(track));

    const counts = (video = null) => {
      const totals = { yes: 0, no: 0, doubt: 0, other: 0, todo: 0 };
      for (const track of tracks.filter((one) => video === null || one.video === video)) {
        const { verdict } = get(track);
        if (verdict) totals[verdict] += 1;
        else if (track.judgeable) totals.todo += 1;
      }
      return totals;
    };

    const judged = (video) => {
      const own = judgeable.filter((track) => track.video === video);
      return { done: own.filter((track) => get(track).verdict !== "").length, total: own.length };
    };

    const written = (value) => (value.verdict === "other" ? value.other : value.verdict ? values[value.verdict][0] : "");

    const line = (track, value) => ({
      video: track.video,
      track: String(track.id),
      start: track.clock,
      startSeconds: track.start,
      verdict: written(value),
      species: value.species,
      startX: track.x,
      startY: track.y,
    });

    const rows = () => {
      const kept = tracks.filter((track) => state.has(keyOf(track))).map((track) => line(track, get(track)));
      return [...kept, ...orphans, ...absent].sort(
        (one, other) => one.video.localeCompare(other.video) || (one.startSeconds ?? 0) - (other.startSeconds ?? 0),
      );
    };

    const orphansOf = (video) =>
      orphans
        .filter((row) => row.video === video)
        .map((row) => {
          const nearest = tracks
            .filter((track) => track.video === video)
            .reduce(
              (best, track) =>
                !best || Math.abs(track.start - row.startSeconds) < Math.abs(best.start - row.startSeconds)
                  ? track
                  : best,
              null,
            );
          const value = entry(row);
          return { row, nearest, word: value.verdict ? wordOf(value) : value.species };
        });

    const nextToSee = (video, fromId) => {
      const own = tracks.filter((track) => track.video === video);
      const start = own.findIndex((track) => track.id === fromId);
      const ordered = [...own.slice(start + 1), ...own.slice(0, Math.max(start, 0))];
      return ordered.find((track) => get(track).verdict === "") ?? null;
    };

    return {
      load,
      changes,
      apply,
      get,
      set,
      isChanged,
      pending,
      drifted,
      markSaved,
      counts,
      kindOf,
      word,
      judged,
      rows,
      orphansOf,
      absentCount: () => absent.length,
      nextToSee,
      track: (video, id) => byKey.get(`${video}\u0000${id}`) ?? null,
      subscribe: (listener) => listeners.push(listener),
    };
  };

  return { create };
})();
