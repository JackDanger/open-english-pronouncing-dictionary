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

await check('click commits a phoneme as .selected (sticky lock)', async () => {
  // After the previous test the chart should have .ph[data-ch="ʃ"]
  // carrying the .selected class.
  const selected = await page.locator('#ipa-chart .ph.selected[data-ch="ʃ"]').count();
  if (selected !== 1) throw new Error(`expected 1 .selected on /ʃ/, got ${selected}`);
});

await check('hover does NOT override a locked sagittal', async () => {
  // /ʃ/ is still selected. Hover a different phoneme; sagittal must
  // stay on /ʃ/.
  await page.dispatchEvent('#ipa-chart .ph[data-ch="m"]', 'mouseover');
  await page.waitForTimeout(80);
  const glyph = await page.locator('#sagittal-glyph').textContent();
  if (glyph !== 'ʃ') throw new Error(`expected sagittal locked on 'ʃ', got '${glyph}'`);
});

await check('second click on same phoneme releases the lock', async () => {
  await page.click('#ipa-chart .ph[data-ch="ʃ"]');
  const stillSelected = await page.locator('#ipa-chart .ph.selected').count();
  if (stillSelected !== 0) throw new Error(`expected 0 .selected after toggle, got ${stillSelected}`);
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

await check('drag a path stop from ɪ → i morphs ship into sheep', async () => {
  // Load `ship` so the path is /ʃ ɪ p/.
  await page.click('.sample-chip[data-word="ship"]');
  await page.waitForTimeout(1500);
  // Synthesize the drag via PointerEvents — playwright's native
  // drag does not work well on SVG.
  const result = await page.evaluate(async () => {
    const stops = document.querySelectorAll('.path-stop');
    const ipaStop = Array.from(stops).find(s => s.dataset.ch === 'ɪ');
    const tile_i = document.querySelector('.ph[data-ch="i"]');
    if (!ipaStop || !tile_i) return { error: 'missing nodes' };
    const r = ipaStop.getBoundingClientRect();
    const t = tile_i.getBoundingClientRect();
    ipaStop.dispatchEvent(new PointerEvent('pointerdown', {bubbles:true, clientX:r.x+r.width/2, clientY:r.y+r.height/2, pointerId:1}));
    document.dispatchEvent(new PointerEvent('pointermove', {bubbles:true, clientX:t.x+t.width/2, clientY:t.y+t.height/2, pointerId:1}));
    document.dispatchEvent(new PointerEvent('pointerup',   {bubbles:true, clientX:t.x+t.width/2, clientY:t.y+t.height/2, pointerId:1}));
    await new Promise(r => setTimeout(r, 200));
    return {
      word:   document.getElementById('word-glyph').textContent,
      crumbVisible: document.getElementById('morph-breadcrumb').classList.contains('visible'),
      crumbText:    document.getElementById('morph-reset').textContent,
    };
  });
  if (result.word !== 'sheep') throw new Error(`expected 'sheep', got '${result.word}'`);
  if (!result.crumbVisible)    throw new Error(`expected morph breadcrumb visible`);
  if (result.crumbText !== 'ship') throw new Error(`crumb says '${result.crumbText}'`);
});

await check('dragging to a non-word surfaces the "no word" indicator', async () => {
  // From `sheep` (the previous state) drag /ʃ/ → /ð/. /ð i p/ is
  // not an English word; the no-word indicator should appear.
  const result = await page.evaluate(async () => {
    const stops = document.querySelectorAll('.path-stop');
    const stop = Array.from(stops).find(s => s.dataset.ch === 'ʃ');
    const tile = document.querySelector('.ph[data-ch="ð"]');
    if (!stop || !tile) return { error: 'missing nodes' };
    const r = stop.getBoundingClientRect();
    const t = tile.getBoundingClientRect();
    stop.dispatchEvent(new PointerEvent('pointerdown', {bubbles:true, clientX:r.x+r.width/2, clientY:r.y+r.height/2, pointerId:1}));
    document.dispatchEvent(new PointerEvent('pointermove', {bubbles:true, clientX:t.x+t.width/2, clientY:t.y+t.height/2, pointerId:1}));
    document.dispatchEvent(new PointerEvent('pointerup',   {bubbles:true, clientX:t.x+t.width/2, clientY:t.y+t.height/2, pointerId:1}));
    await new Promise(r => setTimeout(r, 200));
    return {
      word: document.getElementById('word-glyph').textContent,
      noWord: document.getElementById('morph-noword').classList.contains('visible'),
    };
  });
  if (!result.noWord) throw new Error(`expected no-word indicator on /ðip/`);
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
