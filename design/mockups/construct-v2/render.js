// Renders the construct v2 mocks to PNG at 1280x800, 1x.
// Run from the log worktree: node design/mockups/construct-v2/render.js
const { chromium } = require('playwright');
const path = require('path');
const here = __dirname;
const out = path.resolve(here, '../../../media/2026-11-12/design/construct');
const shots = ['first-floor', 'cube-drag', 'cube-failed'];
(async () => {
  const browser = await chromium.launch();
  const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 });
  const page = await ctx.newPage();
  for (const s of shots.filter(n => !process.argv[2] || n.includes(process.argv[2]))) {
    await page.goto(`file://${path.join(here, s + '.html')}`);
    await page.evaluate(() => document.fonts.ready);
    await page.waitForTimeout(200);
    const file = path.join(out, `mock-${s}.png`);
    await page.screenshot({ path: file });
    console.log(path.relative(process.cwd(), file));
  }
  await browser.close();
})();
