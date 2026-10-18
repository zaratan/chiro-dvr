import { test } from "node:test";
import assert from "node:assert/strict";
import { load } from "./load.mjs";

const { VerdictsTime } = load("time.js");

test("a start is said in words as the evening summary says a length, so a screen reader does not spell out the colons", () => {
  assert.equal(VerdictsTime.aloud(207.27), "3 min 27 s");
  assert.equal(VerdictsTime.aloud(27.9), "27 s");
  assert.equal(VerdictsTime.aloud(3723), "1 h 2 min 3 s");
  assert.equal(VerdictsTime.aloud(3600), "1 h 0 min 0 s");
  assert.equal(VerdictsTime.aloud(0), "0 s");
  assert.equal(VerdictsTime.aloud(-1), "0 s");
});

test("the start said aloud and the start shown drop the same fraction of a second", () => {
  assert.equal(VerdictsTime.clock(59.999), "0:00:59");
  assert.equal(VerdictsTime.aloud(59.999), "59 s");
  assert.equal(VerdictsTime.clock(3725), "1:02:05");
});
