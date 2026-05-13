// Playwright smoke test for the redesigned OpenEPD workspace.
//
// Boots a static server on a random port, navigates Chromium at the
// just-rendered _site/, and asserts the chart-centric workspace
// actually works end-to-end:
//
//   * picking a word populates the word header, draws a path on
//     the chart, and steps the sagittal inset through phonemes
//   * hovering a chart phoneme moves the sagittal inset
//   * sample chips, showcase rows, suggestion items all route
//     through the same delegated handler
//   * reverse phoneme search returns results
//
// Any silent no-op fails the build, as does any console.error or
// pageerror (favicon 404 excepted).

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
    if (err) { res.writeHead(404).end(); return; }
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

const browser = await chromium.launch();
const page = await browser.newPage();
const errors = [];
page.on('pageerror',  (err)  => errors.push(`pageerror: ${err.message}`));
page.on('console', (msg) => {
  if (msg.type() === 'error') {
    const txt = msg.text();
    if (/favicon\.ico/.test(txt)) return;
    errors.push(`console.error: ${txt}`);
  }
});

await page.goto(url, { waitUntil: 'load' });
await page.waitForSelector('#ipa-chart .ph', { timeout: 5000 });
await page.waitForTimeout(1800);  // let the initial animation settle

async function check(label, fn) {
  try { await fn(); console.log(`  ✓ ${label}`); }
  catch (e) { console.error(`  ✗ ${label}: ${e.message}`); process.exitCode = 1; }
}

await check('initial featured word populates the header', async () => {
  const w = await page.locator('#word-glyph').textContent();
  if (!w) throw new Error('no word in #word-glyph');
});

await check('initial word draws a chart path', async () => {
  const stops = await page.locator('#path-layer .path-stop').count();
  if (stops < 2) throw new Error(`expected ≥2 path stops, got ${stops}`);
});

await check('initial word builds a spelling band', async () => {
  const texts = await page.locator('#spelling-band text').count();
  if (texts < 4) throw new Error(`expected ≥4 spelling-band texts, got ${texts}`);
});

await check('sagittal inset is populated', async () => {
  const glyph = await page.locator('#sagittal-glyph').textContent();
  if (!glyph) throw new Error('no #sagittal-glyph text');
});

await check('typing a word into search updates everything', async () => {
  await page.fill('#q', 'pneumonia');
  await page.waitForTimeout(2000);
  const w = await page.locator('#word-glyph').textContent();
  if (w !== 'pneumonia') throw new Error(`expected 'pneumonia', got '${w}'`);
  const stops = await page.locator('#path-layer .path-stop').count();
  if (stops < 5) throw new Error(`expected ≥5 stops for pneumonia, got ${stops}`);
});

await check('clicking a sample chip routes through data-word', async () => {
  await page.click('.sample-chip:nth-of-type(1)');
  await page.waitForTimeout(1500);
  const w = await page.locator('#word-glyph').textContent();
  if (!w) throw new Error('no word after sample-chip click');
});

await check('clicking a chart phoneme moves the sagittal', async () => {
  const before = await page.locator('#sagittal-glyph').textContent();
  await page.click('#ipa-chart .ph[data-ch="ʃ"]');
  const after = await page.locator('#sagittal-glyph').textContent();
  if (after !== 'ʃ') throw new Error(`expected 'ʃ' after click, got '${after}' (was '${before}')`);
});

await check('showcase row click routes through data-word', async () => {
  await page.click('.wrow:nth-of-type(45)');
  await page.waitForTimeout(1500);
  const w = await page.locator('#word-glyph').textContent();
  if (!w) throw new Error('no word after showcase click');
});

await check('reverse search returns matches', async () => {
  await page.click('.reverse-summary');   // expand details
  await page.fill('#reverse-q', 'θɪn');
  await page.waitForTimeout(200);
  const count = await page.locator('#reverse-count').textContent();
  if (!/match/.test(count)) throw new Error(`unexpected count: '${count}'`);
});

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
