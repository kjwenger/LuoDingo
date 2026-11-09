#!/usr/bin/env node
// Loads a page in a real (system) Chrome via Playwright and reports what actually
// happened: console messages, page errors, failed requests, rendered text, and a
// screenshot -- for diagnosing "the page looks broken/blank" without needing a human
// to screenshot it first. Uses the system's installed google-chrome (no bundled
// browser download, keeps this dependency-light).
//
// Usage:
//   node scripts/check-page.mjs <url> [--dark] [--out <screenshot-path>]
//
// Example (reproduced the dark-mode-vs-swagger-ui-react bug this was written for):
//   npm run dev -- -p 3100 &
//   node scripts/check-page.mjs http://localhost:3100/docs --dark --out /tmp/docs.png

import { chromium } from "playwright";

const args = process.argv.slice(2);
const url = args.find((a) => !a.startsWith("--"));
const dark = args.includes("--dark");
const outIndex = args.indexOf("--out");
const outPath = outIndex !== -1 ? args[outIndex + 1] : null;

if (!url) {
  console.error("Usage: node scripts/check-page.mjs <url> [--dark] [--out <path>]");
  process.exit(1);
}

const browser = await chromium.launch({ channel: "chrome" });
const page = await browser.newPage(dark ? { colorScheme: "dark" } : {});

page.on("console", (msg) => console.log(`[console:${msg.type()}]`, msg.text()));
page.on("pageerror", (err) => console.log("[pageerror]", err.message));
page.on("requestfailed", (req) =>
  console.log("[requestfailed]", req.url(), req.failure()?.errorText)
);

await page.goto(url, { waitUntil: "networkidle" });
await page.waitForTimeout(1000);

const bodyText = await page.evaluate(() => document.body.innerText);
const bodyHtmlLength = await page.evaluate(() => document.body.innerHTML.length);

console.log(`--- loaded: ${url} (color-scheme: ${dark ? "dark" : "default"})`);
console.log("--- body innerHTML length:", bodyHtmlLength);
console.log("--- body innerText (first 500 chars) ---");
console.log(bodyText.slice(0, 500));

if (outPath) {
  await page.screenshot({ path: outPath });
  console.log("--- screenshot saved:", outPath);
}

await browser.close();
