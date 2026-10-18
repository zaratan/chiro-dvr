import { chromium } from "playwright-core";
import { spawn } from "node:child_process";
import fs from "node:fs";
const options = JSON.parse(process.argv[2]);
const port = 8340 + Math.floor(Math.random() * 50);
const server = spawn("node", ["web/server.mjs", ".", String(port), options.isolated ? "isolated" : "plain"], { stdio: "inherit" });
await new Promise((r) => setTimeout(r, 300));
const browser = await chromium.launch({ args: options.args ?? [] });
let peakRss = 0;
const sample = () => {
  const parents = new Map();
  const rssOf = new Map();
  for (const pid of fs.readdirSync("/proc").filter((name) => /^\d+$/.test(name))) {
    try {
      const status = fs.readFileSync(`/proc/${pid}/status`, "utf8");
      parents.set(pid, status.match(/PPid:\s+(\d+)/)[1]);
      const rss = status.match(/VmRSS:\s+(\d+)/);
      rssOf.set(pid, rss ? Number(rss[1]) * 1024 : 0);
    } catch {}
  }
  let total = 0;
  for (const [pid, rss] of rssOf) {
    if (pid === String(process.pid) || pid === String(server.pid)) continue;
    for (let at = parents.get(pid); at && at !== "0" && at !== "1"; at = parents.get(at)) {
      if (at === String(process.pid)) { total += rss; break; }
    }
  }
  peakRss = Math.max(peakRss, total);
};
const sampler = setInterval(sample, 100);
try {
  const page = await browser.newPage();
  page.on("console", (m) => console.error("console:", m.text()));
  await page.goto(`http://127.0.0.1:${port}/web/${options.page ?? "page.html"}`);
  const isolated = await page.evaluate(() => window.isolated);
  const result = await page.evaluate((o) => window.run(o), options.run);
  if (options.download) {
    const [download] = await Promise.all([page.waitForEvent("download"), page.click("#save")]);
    await download.saveAs(options.download);
  }
  const { hashes = [], result: text = "", ...summary } = result;
  if (options.out) { fs.writeFileSync(options.out + ".txt", text); fs.writeFileSync(options.out + ".hashes", hashes.join("\n") + (hashes.length ? "\n" : "")); }
  clearInterval(sampler);
  console.log(JSON.stringify({ isolated, version: browser.version(), peakChromeRssMB: Math.round(peakRss / 1e6), ...summary }));
} finally {
  await browser.close();
  server.kill();
}
