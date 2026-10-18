import { test } from "node:test";
import assert from "node:assert/strict";
import { rules } from "./css.mjs";

test("a text kept for screen readers is left out of a selection, so a copied row holds only what is shown", () => {
  const hidden = rules.find(({ selector }) => selector === ".sr");
  assert.ok(hidden);
  assert.match(hidden.body, /(^|[\s;])user-select:\s*none/);
  assert.match(hidden.body, /-webkit-user-select:\s*none/);
});
