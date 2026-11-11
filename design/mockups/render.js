// Renders the HUD mockups to PNG with Playwright.
// Run from the log worktree: node design/mockups/render.js [filter]
const { chromium } = require('playwright');
const path = require('path');

const here = __dirname;
const out = path.resolve(here, '../../media/2026-11-11/construct-hud');
const filter = process.argv[2] || '';

const shots = [];
for (const theme of ['light', 'dark']) {
  for (const state of ['walk', 'list', 'question', 'chat', 'cube', 'help', 'help-search']) {
    shots.push({ name: `construct-${state}-${theme}`, page: 'construct.html', q: `state=${state}&theme=${theme}` });
  }
  shots.push({ name: `keys-mc-${theme}`, page: 'keys-mc.html', q: `theme=${theme}` });
}
shots.push({ name: 'keys-xp', page: 'keys-xp.html', q: '' });

(async () => {
  const browser = await chromium.launch();
  for (const scale of [1, 2]) {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 }, deviceScaleFactor: scale });
    const page = await ctx.newPage();
    for (const s of shots) {
      if (filter && !s.name.includes(filter)) continue;
      await page.goto(`file://${path.join(here, s.page)}?${s.q}`);
      await page.evaluate(() => document.fonts.ready);
      await page.waitForTimeout(150);
      const file = path.join(out, `${s.name}${scale === 2 ? '-2x' : ''}.png`);
      await page.screenshot({ path: file });
      console.log(path.relative(process.cwd(), file));
    }
    await ctx.close();
  }
  await browser.close();
})();
