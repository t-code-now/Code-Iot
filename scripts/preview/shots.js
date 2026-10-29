// Screenshots of a preview build, phone and desktop, in Hebrew.
//   node scripts/preview/mock-api.js <path-to-dist> 4100   (in another shell)
//   node scripts/preview/shots.js /login /terms /lesson/py-1-2
// Writes m_<page>.png (phone 400px) and d_<page>.png (desktop 1366px) to the
// current folder. Add `--signed-in` to act as a signed-in user.
const { chromium } = require('playwright');
const args = process.argv.slice(2);
const signedIn = args.includes('--signed-in');
const pages = args.filter((a) => !a.startsWith('--'));
(async () => {
  const b = await chromium.launch();
  const sizes = { m: { width: 400, height: 860, scale: 2 }, d: { width: 1366, height: 820, scale: 1 } };
  for (const [k, s] of Object.entries(sizes)) {
    const ctx = await b.newContext({ viewport: { width: s.width, height: s.height }, deviceScaleFactor: s.scale });
    await ctx.addInitScript((tok) => {
      localStorage.setItem('cs.lang', '"he"');
      localStorage.setItem('cs.guest', 'true');
      if (tok) localStorage.setItem('cs.token', '"tok"');
    }, signedIn);
    const p = await ctx.newPage();
    p.on('pageerror', (e) => console.log('page error:', e.message));
    for (const r of pages.length ? pages : ['/']) {
      await p.goto('http://127.0.0.1:4100' + r);
      await p.waitForTimeout(1200);
      const name = r.replace(/^\//, '').replace(/\//g, '_') || 'home';
      await p.screenshot({ path: `${k}_${name}.png` });
    }
    await ctx.close();
  }
  await b.close();
})();
