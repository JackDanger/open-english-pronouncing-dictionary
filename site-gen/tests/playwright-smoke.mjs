// Playwright smoke test for the rendered OpenEPD site.
//
// Run in CI after `site-gen` writes _site/. We launch a tiny static
// server on a random port, navigate Playwright at it, click one of
// every interactive element kind, and assert nothing throws.
//
//   npm i playwright
//   npx playwright install chromium
//   node site-gen/tests/playwright-smoke.mjs ./_site

import http from 'node:http';
import fs from 'node:fs';
import path from 'node:path';
import { chromium } from 'playwright';

const siteDir = process.argv[2] || '_site';
const indexPath = path.join(siteDir, 'index.html');
if (!fs.existsSync(indexPath)) {
  console.error(`No ${indexPath}; nothing to smoke test.`);
  process.exit(2);
}

// ── Tiny static server ─────────────────────────────────────────────
const server = http.createServer((req, res) => {
  const url = req.url === '/' ? '/index.html' : req.url.split('?')[0];
  const file = path.join(siteDir, url);
  fs.readFile(file, (err, data) => {
    if (err) {
      res.writeHead(404).end();
      return;
    }
    const type = file.endsWith('.html') ? 'text/html; charset=utf-8' :
                 file.endsWith('.js')   ? 'application/javascript' :
                 file.endsWith('.css')  ? 'text/css' :
                 'application/octet-stream';
    res.writeHead(200, { 'content-type': type }).end(data);
  });
});
await new Promise(r => server.listen(0, '127.0.0.1', r));
const { port } = server.address();
const url = `http://127.0.0.1:${port}/`;
console.log(`serving ${siteDir} on ${url}`);

// ── Drive the page ─────────────────────────────────────────────────
const browser = await chromium.launch();
const page = await browser.newPage();

// Collect every page error and every console error so we can fail
// loud on either kind.
const errors = [];
page.on('pageerror',  (err)  => errors.push(`pageerror: ${err.message}`));
page.on('console', (msg) => {
  if (msg.type() === 'error') {
    const txt = msg.text();
    // The favicon 404 is harmless — every minimal GitHub Pages site
    // has one. Skip it; we want REAL JS errors.
    if (/favicon\.ico/.test(txt)) return;
    errors.push(`console.error: ${txt}`);
  }
});

await page.goto(url, { waitUntil: 'load' });
await page.waitForSelector('.wrow', { timeout: 5000 });

// Drive every interactive kind. We assert each click changes state in
// the expected DOM region rather than just "didn't throw", so a silent
// no-op is caught too.

async function check(label, fn) {
  try {
    await fn();
    console.log(`  ✓ ${label}`);
  } catch (e) {
    console.error(`  ✗ ${label}: ${e.message}`);
    process.exitCode = 1;
  }
}

await check('showcase row click → result card', async () => {
  await page.click('.wrow:nth-of-type(45)');
  const word = await page.locator('.result-word').first().textContent();
  if (!word) throw new Error('no .result-word after click');
});

await check('phoneme token click → phoneme panel', async () => {
  await page.click('.phoneme-token >> nth=3');
  const glyph = await page.locator('.panel-glyph').textContent();
  if (!glyph) throw new Error('no .panel-glyph after click');
});

await check('panel-word click → new result', async () => {
  const before = await page.locator('.result-word').textContent();
  await page.click('.panel-word >> nth=0');
  const after = await page.locator('.result-word').textContent();
  if (before === after) throw new Error('result-word did not change');
});

await check('phoneme universe tile click → panel updates', async () => {
  await page.click('.ptile[data-ch="ʃ"]');
  // selectPhonemeFromUniverse has a 350ms scroll delay then renders
  await page.waitForTimeout(600);
  const name = await page.locator('.panel-name').textContent();
  if (!/SH/.test(name)) throw new Error(`unexpected panel-name: ${name}`);
});

await check('heatmap cell click → distance panel', async () => {
  await page.click('.heatmap-cell[data-row="5"][data-col="12"]');
  const values = await page.locator('#distance-panel .dp-value').allTextContents();
  if (values.length < 2) throw new Error(`expected 2 dp-values, got ${values.length}`);
});

await check('minimal-pair row click → result', async () => {
  const before = await page.locator('.result-word').textContent();
  await page.click('.pair-row >> nth=0');
  const after = await page.locator('.result-word').textContent();
  if (before === after) throw new Error('result-word did not change');
});

await check('reverse phoneme search → results', async () => {
  await page.fill('#reverse-q', 'θɪn');
  await page.waitForTimeout(200);
  const count = await page.locator('#reverse-count').textContent();
  if (!/match/.test(count)) throw new Error(`unexpected reverse-count: ${count}`);
});

await check('try-word click → result', async () => {
  await page.click('.try-word');
  const word = await page.locator('.result-word').textContent();
  if (!word) throw new Error('no result-word after try-word click');
});

// Done.
if (errors.length) {
  console.error('\nJS errors during smoke test:');
  for (const e of errors) console.error('  ' + e);
  process.exitCode = 1;
} else {
  console.log('\nNo console / page errors.');
}

await browser.close();
server.close();
process.exit(process.exitCode ?? 0);
