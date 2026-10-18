import { readFileSync } from "node:fs";
import vm from "node:vm";

export const load = (...files) => {
  for (const file of files) {
    vm.runInThisContext(readFileSync(new URL(`../${file}`, import.meta.url), "utf8"), { filename: file });
  }
  return vm.runInThisContext(
    `({ VerdictsCsv: typeof VerdictsCsv === "undefined" ? undefined : VerdictsCsv, VerdictsStore: typeof VerdictsStore === "undefined" ? undefined : VerdictsStore, VerdictsHits: typeof VerdictsHits === "undefined" ? undefined : VerdictsHits, VerdictsFilter: typeof VerdictsFilter === "undefined" ? undefined : VerdictsFilter, VerdictsTime: typeof VerdictsTime === "undefined" ? undefined : VerdictsTime, VerdictsOverlay: typeof VerdictsOverlay === "undefined" ? undefined : VerdictsOverlay })`,
  );
};
