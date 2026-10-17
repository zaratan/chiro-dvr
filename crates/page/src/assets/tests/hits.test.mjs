import { test } from "node:test";
import assert from "node:assert/strict";
import { load } from "./load.mjs";

const { VerdictsHits } = load("hits.js");

const shape = (id, line, { head = [], dot = null } = {}) => ({
  id,
  lines: [VerdictsHits.points(line)].filter((one) => one.length > 0),
  polygons: head.length > 0 ? [head] : [],
  circles: dot ? [dot] : [],
});

const crossing = () => [
  shape(12, "100,100 300,100", { dot: { x: 320, y: 100, r: 10 } }),
  shape(13, "100,104 300,160"),
  shape(4, "100,400 300,400", { head: [[300, 390], [320, 400], [300, 410]] }),
];

const OPTIONS = { reach: 12, current: null };

test("the points of a trace are read from the text of the page, skipping what is not a pair of numbers", () => {
  assert.deepEqual(VerdictsHits.points(" 10,20  30,40 x,5 7 "), [
    [10, 20],
    [30, 40],
  ]);
});

test("a point is as far from a trace as from its nearest stroke, arrow head or dot", () => {
  const [twelve, , four] = crossing();
  assert.equal(VerdictsHits.distance([200, 106], twelve), 6);
  assert.equal(VerdictsHits.distance([325, 100], twelve), 0);
  assert.equal(VerdictsHits.distance([305, 400], four), 0);
  assert.equal(VerdictsHits.distance([330, 400], four), 10);
  assert.equal(VerdictsHits.distance([100, 90], twelve), 10);
});

test("a track that never moved is a single point, and a repeated point is no segment", () => {
  assert.equal(VerdictsHits.distance([3, 4], shape(1, "0,0")), 5);
  assert.equal(VerdictsHits.distance([3, 4], shape(1, "0,0 0,0")), 5);
});

test("a track with nothing drawn is never reached", () => {
  assert.equal(VerdictsHits.distance([0, 0], shape(1, "")), Infinity);
  assert.deepEqual(VerdictsHits.reached([0, 0], [shape(1, "")], 12), []);
});

test("a click reaches only the traces within its zone, the nearest first", () => {
  assert.deepEqual(VerdictsHits.reached([100, 101], crossing(), 12), [12, 13]);
  assert.deepEqual(VerdictsHits.reached([100, 104], crossing(), 12), [13, 12]);
  assert.deepEqual(VerdictsHits.reached([200, 250], crossing(), 12), []);
});

test("a click chooses the trace that passes nearest, and nothing far from every trace", () => {
  assert.equal(VerdictsHits.choose([100, 101], crossing(), OPTIONS), 12);
  assert.equal(VerdictsHits.choose([100, 104], crossing(), OPTIONS), 13);
  assert.equal(VerdictsHits.choose([200, 250], crossing(), OPTIONS), null);
});

test("a second click where the judged track is reached chooses the other overlapping trace, then comes back", () => {
  const first = VerdictsHits.choose([100, 101], crossing(), OPTIONS);
  const second = VerdictsHits.choose([101, 102], crossing(), { ...OPTIONS, current: first });
  const third = VerdictsHits.choose([100, 101], crossing(), { ...OPTIONS, current: second });
  assert.deepEqual([first, second, third], [12, 13, 12]);
});

test("three overlapping traces are each reached in turn", () => {
  const bundle = [shape(1, "0,0 100,0"), shape(2, "0,2 100,2"), shape(3, "0,4 100,4")];
  const chosen = [];
  let current = null;
  for (let click = 0; click < 4; click += 1) {
    current = VerdictsHits.choose([50, 0], bundle, { reach: 12, current });
    chosen.push(current);
  }
  assert.deepEqual(chosen, [1, 2, 3, 1]);
});

test("a click where the judged track is out of reach chooses the nearest again", () => {
  assert.equal(VerdictsHits.choose([100, 101], crossing(), { ...OPTIONS, current: 4 }), 12);
});

test("a click on the judged track alone keeps it", () => {
  assert.equal(VerdictsHits.choose([200, 400], crossing(), { ...OPTIONS, current: 4 }), 4);
});

test("two traces at the same distance are taken in the order of their numbers", () => {
  const twins = [shape(9, "0,10 100,10"), shape(3, "0,-10 100,-10")];
  assert.deepEqual(VerdictsHits.reached([50, 0], twins, 12), [3, 9]);
});
