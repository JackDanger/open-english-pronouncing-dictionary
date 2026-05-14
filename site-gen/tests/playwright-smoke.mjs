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

await check('selecting a phoneme mid-animation cancels the animation walk', async () => {
  // Pick a long word so the path animation takes a while, then click
  // a chart phoneme almost immediately. The user's click must win:
  // selectedCh stays put and the sagittal isn't clobbered by the
  // tail of the animation.
  await page.fill('#q', 'phonetics');
  await page.waitForTimeout(100);  // animation just started
  await page.click('#ipa-chart .ph[data-ch="ʒ"]');
  await page.waitForTimeout(2500); // well past the animation's normal duration
  const glyph = await page.locator('#sagittal-glyph').textContent();
  if (glyph !== 'ʒ') throw new Error(`expected sagittal still on 'ʒ' after animation finishes, got '${glyph}'`);
  const stillSelected = await page.locator('#ipa-chart .ph.selected[data-ch="ʒ"]').count();
  if (stillSelected !== 1) throw new Error(`expected /ʒ/ still .selected`);
});

/* `realDrag` uses locator.dragTo — the actual hit-tested drag a user
 * would make. Synthetic dispatchEvent() bypasses hit-testing and
 * would pass even if the path stop were buried under a phoneme tile
 * (a real bug we previously shipped). */
async function realDrag(fromSel, toSel) {
  await page.locator(fromSel).dragTo(page.locator(toSel));
  await page.waitForTimeout(200);
}

await check('drag a path stop from ɪ → i morphs ship into sheep', async () => {
  await page.click('.sample-chip[data-word="ship"]');
  await page.waitForTimeout(1500);
  await realDrag('.path-stop[data-ch="ɪ"]', '.ph[data-ch="i"]');
  const word = await page.locator('#word-glyph').textContent();
  if (word !== 'sheep') throw new Error(`expected 'sheep', got '${word}'`);
  const crumbVisible = await page.locator('#morph-breadcrumb.visible').count();
  if (crumbVisible !== 1) throw new Error('expected morph breadcrumb visible');
  const crumbText = await page.locator('#morph-reset').textContent();
  if (crumbText !== 'ship') throw new Error(`crumb says '${crumbText}'`);
});

await check('starting a drag lights up valid morph-target phonemes', async () => {
  await page.click('.sample-chip[data-word="ship"]');
  await page.waitForTimeout(1500);
  // Begin a drag on /ʃ/ (a consonant with many minimal-pair
  // neighbours — hip, sip, tip, …). Don't release; check the
  // lit landscape.
  const result = await page.evaluate(async () => {
    const stop = Array.from(document.querySelectorAll('.path-stop')).find(s => s.dataset.ch === 'ʃ');
    const r = stop.getBoundingClientRect();
    stop.dispatchEvent(new PointerEvent('pointerdown', { bubbles:true, clientX:r.x+r.width/2, clientY:r.y+r.height/2, pointerId:1 }));
    await new Promise(r => setTimeout(r, 80));
    const lit  = Array.from(document.querySelectorAll('.ph.morph-target')).map(el => el.dataset.ch);
    const labels = Array.from(document.querySelectorAll('.morph-label')).map(el => el.textContent);
    const origin = document.querySelector('.ph.morph-origin')?.dataset?.ch;
    // Release without moving — should clear everything.
    document.dispatchEvent(new PointerEvent('pointerup', { bubbles:true, pointerId:1 }));
    await new Promise(r => setTimeout(r, 80));
    const litAfter = document.querySelectorAll('.ph.morph-target').length;
    return { lit, labels, origin, litAfter };
  });
  if (result.origin !== 'ʃ') throw new Error(`expected origin 'ʃ', got '${result.origin}'`);
  if (result.lit.length < 5) throw new Error(`expected ≥5 lit moves, got ${result.lit.length}`);
  if (!result.labels.some(l => l === 'hip' || l === 'sip' || l === 'tip')) {
    throw new Error(`expected at least one common minimal-pair word in labels, saw ${result.labels.slice(0,5).join(', ')}`);
  }
  if (result.litAfter !== 0) throw new Error('lit state should clear on release');
});

await check('drag releases on a dim tile snap to a valid target instead', async () => {
  // Try to drag /ʃ/ toward /ð/. /ðip/ isn't a word, so /ð/ isn't a
  // lit target. The drag should snap to the nearest VALID target
  // instead (whichever lit tile is closest to ð's chart position).
  await page.click('.sample-chip[data-word="ship"]');
  await page.waitForTimeout(1500);
  await realDrag('.path-stop[data-ch="ʃ"]', '.ph[data-ch="ð"]');
  // The post-drag word must still be a real English word (i.e. the
  // "no English word" indicator stays hidden).
  const noWord = await page.locator('#morph-noword.visible').count();
  if (noWord !== 0) throw new Error('lit-moves drag should never land on a non-word');
  const word = await page.locator('#word-glyph').textContent();
  if (!word || word === '⟨—⟩') throw new Error(`expected a real word after drag, got '${word}'`);
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
