import { chromium } from "playwright-core";
import fs from "node:fs";
const [folder, saveAs] = process.argv.slice(2);
const browser = await chromium.launch();
try {
  const context = await browser.newContext({ acceptDownloads: true });
  const page = await context.newPage();
  const errors = [];
  page.on("pageerror", (error) => errors.push(String(error)));
  page.on("console", (m) => { if (m.type() === "error") errors.push(m.text()); });
  await page.goto("file://" + fs.realpathSync(folder) + "/resultats.html");
  const dots = await page.locator('button.dot[aria-label^="Juger la piste"]').count();
  await page.locator('button.dot[aria-label="Juger la piste 1"]').first().click();
  const video = page.locator(".judge video");
  await page.waitForFunction(() => { const v = document.querySelector(".judge video"); return v && v.readyState >= 2; }, null, { timeout: 15000 });
  const clip = await video.evaluate((v) => ({ src: v.currentSrc.split("/").slice(-2).join("/"), duration: v.duration, width: v.videoWidth, height: v.videoHeight, error: v.error?.code ?? null }));
  await page.keyboard.press("o");
  await page.locator(".judge .yes[aria-pressed=true]").waitFor({ timeout: 5000 });
  const [download] = await Promise.all([page.waitForEvent("download"), page.locator("button.download").click()]);
  await download.saveAs(saveAs);
  console.log(JSON.stringify({ dots, clip, suggested: download.suggestedFilename(), errors }));
} finally {
  await browser.close();
}
