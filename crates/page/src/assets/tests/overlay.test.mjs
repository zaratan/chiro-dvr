import { test } from "node:test";
import assert from "node:assert/strict";
import { load } from "./load.mjs";

const { VerdictsStore, VerdictsOverlay } = load("store.js", "overlay.js");
const VALUES = { yes: ["oui"], no: ["non"], doubt: ["?"] };
const say = (key, values = {}) => [key, ...Object.entries(values).map(([name, value]) => `${name}=${value}`)].join(" ");
const track = (id) => ({ video: "v", id, start: id * 10, x: null, y: null, judgeable: true, clock: "0:00:10", aloud: `${id * 10} s` });

const named = (verdict) => {
  const store = VerdictsStore.create({ tracks: [track(3)], values: VALUES, say });
  if (verdict) store.load([{ video: "v", track: "3", start: "", startSeconds: 30, verdict, species: "", startX: null, startY: null }]);
  return VerdictsOverlay.named(store, say, store.track("v", 3));
};

test("a trace says its verdict in plain words, since a bare yes or no means nothing away from the verdict buttons", () => {
  assert.match(named("oui"), /^trace-yes /);
  assert.match(named("non"), /^trace-no /);
  assert.match(named("?"), /^trace-doubt /);
  assert.match(named(""), /^trace-to-see /);
});

test("a trace noted with another word says that word", () => {
  assert.match(named("pipistrelle"), /^trace-other .*value=pipistrelle/);
});

test("a trace says its start in words, not as the clock shown", () => {
  assert.match(named("oui"), /track=3 start=30 s/);
});
