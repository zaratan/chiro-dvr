import { test } from "node:test";
import assert from "node:assert/strict";
import { rules } from "./css.mjs";

test("the thumb fades the other tracks only while no trace is hovered, so the hovered one stands out as on the big picture", () => {
  const fading = rules.filter(({ selector, body }) => selector.includes(".thumb ") && /opacity/.test(body));
  assert.notEqual(fading.length, 0);
  for (const { selector } of fading) assert.match(selector, /\.scene:not\(\.hovering\)/);
  assert.ok(rules.some(({ selector }) => selector === ".scene.hovering [data-id]:not(.hot)"));
});
