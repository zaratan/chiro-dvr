import { test } from "node:test";
import assert from "node:assert/strict";
import { load } from "./load.mjs";

const { VerdictsStore } = load("store.js");
const VALUES = { yes: ["oui", "yes"], no: ["non", "no"], doubt: ["?"] };

const track = (video, id, start, x = null, y = null) => ({ video, id, start, x, y, judgeable: true, clock: `0:00:${String(Math.floor(start)).padStart(2, "0")}` });
const row = (video, startSeconds, verdict, extra = {}) => ({
  video,
  track: "",
  start: "",
  startSeconds,
  verdict,
  species: "",
  startX: null,
  startY: null,
  ...extra,
});

const evening = () => [track("v", 1, 4.8, 100, 100), track("v", 2, 4.85, 900, 500), track("v", 3, 30.0), track("w", 1, 10.0)];

test("a verdict finds its track again when the start moved by less than a fifth of a second", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([row("v", 30.17, "oui")]);
  assert.equal(store.get(store.track("v", 3)).verdict, "yes");
  assert.equal(store.orphansOf("v").length, 0);
});

test("two tracks starting together are told apart by their starting point", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([row("v", 4.8, "non", { startX: 905, startY: 498 }), row("v", 4.82, "oui", { startX: 98, startY: 101 })]);
  assert.equal(store.get(store.track("v", 1)).verdict, "yes");
  assert.equal(store.get(store.track("v", 2)).verdict, "no");
});

test("a verdict whose track is gone is kept apart with the nearest track and written back", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([row("v", 12.0, "oui", { track: "7" })]);
  const [lost] = store.orphansOf("v");
  assert.equal(lost.row.track, "7");
  assert.equal(lost.nearest.id, 2);
  assert.deepEqual(store.rows().map((one) => one.startSeconds), [12.0]);
});

test("a verdict of a video absent from the page is counted and written back as it was", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  const elsewhere = row("autre", 3.0, "?", { species: "murin" });
  store.load([elsewhere]);
  assert.equal(store.absentCount(), 1);
  assert.deepEqual(store.rows(), [elsewhere]);
});

test("a value the page does not know is kept and written back unchanged", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([row("w", 10.0, "Chauve-souris !")]);
  const value = store.get(store.track("w", 1));
  assert.deepEqual([value.verdict, value.other], ["other", "Chauve-souris !"]);
  assert.equal(store.rows()[0].verdict, "Chauve-souris !");
});

test("a verdict is read whatever its case and spaces and written in the words of the page", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([row("w", 10.0, "  OUI ")]);
  assert.equal(store.rows()[0].verdict, "oui");
});

test("the counts give each verdict and the tracks still to see", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([row("v", 4.8, "oui", { startX: 100, startY: 100 }), row("w", 10.0, "?")]);
  assert.deepEqual(store.counts(), { yes: 1, no: 0, doubt: 1, other: 0, todo: 2 });
  assert.deepEqual(store.judged("v"), { done: 1, total: 3 });
});

test("the tracks of a video without viewer are neither to see nor judged, yet an earlier verdict on them counts and is written back", () => {
  const crowded = [1, 2, 3].map((id) => ({ ...track("x", id, 10.0 * id), judgeable: false }));
  const store = VerdictsStore.create({ tracks: [...evening(), ...crowded], values: VALUES });
  store.load([row("x", 20.0, "oui")]);
  assert.deepEqual(store.counts(), { yes: 1, no: 0, doubt: 0, other: 0, todo: 4 });
  assert.deepEqual(store.judged("x"), { done: 0, total: 0 });
  assert.equal(store.orphansOf("x").length, 0);
  assert.deepEqual(
    store.rows().map((one) => [one.video, one.track, one.verdict]),
    [["x", "2", "oui"]],
  );
});

test("a change is pending until downloaded and a change undone is no longer pending", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([]);
  const first = store.track("v", 1);
  store.set(first, { verdict: "yes" });
  assert.equal(store.pending(), 1);
  store.set(first, { verdict: "" });
  assert.equal(store.pending(), 0);
  store.set(first, { species: "murin" });
  store.markSaved();
  assert.equal(store.pending(), 0);
});

test("only the tracks with a verdict or a species are written, in the order of the videos and starts", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([]);
  store.set(store.track("w", 1), { verdict: "no" });
  store.set(store.track("v", 3), { species: "pipistrelle" });
  assert.deepEqual(
    store.rows().map((one) => [one.video, one.track, one.verdict, one.species]),
    [["v", "3", "", "pipistrelle"], ["w", "1", "non", ""]],
  );
});

test("a draft written back restores the same verdicts on the same tracks", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([]);
  store.set(store.track("v", 2), { verdict: "doubt", species: "noctule" });
  const draft = store.changes();
  const reopened = VerdictsStore.create({ tracks: evening(), values: VALUES });
  reopened.load([]);
  reopened.apply(draft);
  assert.deepEqual(reopened.get(reopened.track("v", 2)), { verdict: "doubt", other: "", species: "noctule" });
  assert.equal(reopened.pending(), 1);
});

test("the next track to see skips the judged ones and comes back to the start of the video", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([]);
  store.set(store.track("v", 3), { verdict: "yes" });
  assert.equal(store.nextToSee("v", 2).id, 1);
  store.set(store.track("v", 1), { verdict: "no" });
  store.set(store.track("v", 2), { verdict: "no" });
  assert.equal(store.nextToSee("v", 2), null);
});

test("a download clears what is pending but the verdicts still differ from the file the page read", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([]);
  store.set(store.track("v", 1), { verdict: "yes" });
  store.markSaved();
  assert.equal(store.pending(), 0);
  assert.equal(store.drifted(), true);
});

test("a draft keeps its verdict even when its track is gone, instead of losing it", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([]);
  store.apply([row("v", 55.0, "oui"), row("ailleurs", 1.0, "non")]);
  assert.deepEqual(store.rows().map((one) => one.video), ["ailleurs", "v"]);
  assert.equal(store.orphansOf("v").length, 1);
});

test("taking a draft back keeps the verdicts of a newer file that the draft did not touch", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([]);
  store.set(store.track("v", 3), { verdict: "no" });
  const changes = store.changes();
  const reopened = VerdictsStore.create({ tracks: evening(), values: VALUES });
  reopened.load([row("w", 10.0, "oui")]);
  reopened.apply(changes);
  assert.equal(reopened.get(reopened.track("w", 1)).verdict, "yes");
  assert.equal(reopened.get(reopened.track("v", 3)).verdict, "no");
});

test("a verdict cleared in a draft is cleared again when the draft is taken back", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([row("w", 10.0, "oui")]);
  store.set(store.track("w", 1), { verdict: "" });
  const reopened = VerdictsStore.create({ tracks: evening(), values: VALUES });
  reopened.load([row("w", 10.0, "oui")]);
  reopened.apply(store.changes());
  assert.equal(reopened.get(reopened.track("w", 1)).verdict, "");
});

test("a verdict written in the other language of the page is still understood", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  store.load([row("w", 10.0, "Yes")]);
  assert.equal(store.get(store.track("w", 1)).verdict, "yes");
  assert.equal(store.rows()[0].verdict, "oui");
});

test("a line whose start cannot be read is kept as it was and written back", () => {
  const store = VerdictsStore.create({ tracks: evening(), values: VALUES });
  const odd = row("v", null, "oui", { species: "noctule" });
  store.load([odd]);
  assert.deepEqual(store.rows(), [odd]);
});
