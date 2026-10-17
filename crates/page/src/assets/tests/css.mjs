import { readFileSync } from "node:fs";

export const rules = readFileSync(new URL("../page.css", import.meta.url), "utf8")
  .replace(/@[^{;]+\{/g, "")
  .split("}")
  .map((rule) => rule.split("{"))
  .filter((parts) => parts.length === 2)
  .flatMap(([selectors, body]) => selectors.split(",").map((selector) => ({ selector: selector.trim(), body })));
