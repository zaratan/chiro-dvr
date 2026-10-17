import { test } from "node:test";
import assert from "node:assert/strict";
import { load } from "./load.mjs";

const { VerdictsFilter } = load("filter.js");

test("every track passes the filter of all tracks, whatever its verdict", () => {
  for (const kind of ["todo", "yes", "no", "doubt", "other"]) assert.ok(VerdictsFilter.passes(kind, "all", false));
});

test("a filter by verdict lets through only the tracks of that verdict", () => {
  assert.ok(VerdictsFilter.passes("todo", "todo", false));
  assert.ok(!VerdictsFilter.passes("yes", "todo", false));
  assert.ok(VerdictsFilter.passes("no", "no", false));
  assert.ok(!VerdictsFilter.passes("other", "doubt", false));
});

test("the track being judged stays shown whatever the filter", () => {
  for (const chosen of ["todo", "yes", "no", "doubt"]) assert.ok(VerdictsFilter.passes("other", chosen, true));
});

test("the filter of other words lets through only the tracks noted with another word", () => {
  assert.ok(VerdictsFilter.passes("other", "other", false));
  assert.ok(!VerdictsFilter.passes("doubt", "other", false));
  assert.ok(!VerdictsFilter.passes("todo", "other", false));
});
