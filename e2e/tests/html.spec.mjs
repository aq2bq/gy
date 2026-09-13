import {test, expect} from '@playwright/test';
import {readFileSync, writeFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
import os from 'node:os';
import {dir, fixture, mouse, open, enter, scale} from '../helpers.mjs';

async function clusterContract(page) {
  await mouse(page, page.locator('#svg text.nlabel').filter({hasText: 's0 / decision'}));
  await expect(page.locator('#clusterPanel'), 'cluster label must open its member list').toHaveClass(/on/);
  await expect(page.locator('#clusterPanel h3')).toContainText('s0 / decision');
  expect(await page.locator('#clusterPanel li').count()).toBe(175);
}
async function refitContract(page) {
  for (let i = 0; i < 6; i++) await mouse(page, '#zout');
  expect(await scale(page)).toBeLessThan(1);
  for (let i = 0; i < 6; i++) {
    const box = await page.locator('#svg').evaluate(el => { const r=el.getBoundingClientRect(); return {x:r.x,y:r.y,width:r.width,height:r.height}; });
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    await page.mouse.down();
    await page.mouse.move(box.x + box.width / 2 + 250, box.y + box.height / 2);
    await page.mouse.up();
  }
  await enter(page, 'D-1');
  expect(await page.locator('[data-node-id]').count()).toBeGreaterThan(0);
  expect(await scale(page), 'changed target must refit to readable scale').toBeGreaterThanOrEqual(1);
  await expect(page.locator('[data-node-id="D-1"]')).toBeInViewport();
  const labels = await page.locator('[data-node-id] .nlabel').evaluateAll(es => es.map(e => e.getBoundingClientRect().height));
  expect(labels.length).toBeGreaterThan(0);
  expect(Math.min(...labels), 'labels must be readable, not merely non-overlapping').toBeGreaterThanOrEqual(11);
}

test.beforeEach(async ({page}) => {
  const errors = [];
  page.on('pageerror', e => errors.push(e.message));
  const requests = [];
  page.on('request', r => { if (/^https?:/.test(r.url())) requests.push(r.url()); });
  await page.route('https://**/*', route => route.abort());
  await page.route('http://**/*', route => route.abort());
  page._gyErrors = errors; page._gyRequests = requests;
});
test.afterEach(async ({page}) => {
  expect(page._gyErrors).toEqual([]);
  expect(page._gyRequests, 'standalone HTML must not request network assets').toEqual([]);
});

test('cluster labels receive mouse input', async ({page}) => {
  await open(page); await clusterContract(page);
});
test('display target changes refit after manual zoom', async ({page}) => {
  await open(page); await refitContract(page);
});
test('injected label interception is detected by the same contract', async ({page}, info) => {
  await open(page);
  await page.addStyleTag({content: 'svg .nlabel { pointer-events: auto !important; }'});
  await expect(clusterContract(page)).rejects.toThrow(/cluster label must open/);
  await info.attach('injected-defect', {body: 'The unchanged cluster contract rejected pointer-events:auto.', contentType: 'text/plain'});
});
test('injected missing refit is detected by the same contract', async ({page}, info) => {
  const html = readFileSync(resolve(dir, 'large/gy.html'), 'utf8');
  const anchor = 'function fitTransform() {';
  expect(html.split(anchor).length - 1).toBe(1);
  // Private test copy only: allow the initial fit, drop later automatic fits.
  const mutant = html.replace(anchor, 'let testFitCalls = 0;\nfunction fitTransform() { if (++testFitCalls > 1) return;');
  const path = info.outputPath('missing-refit.html');
  writeFileSync(path, mutant);
  await page.goto(pathToFileURL(path).href);
  await expect(refitContract(page)).rejects.toThrow(/changed target must refit/);
  await expect(page.locator('[data-node-id="D-1"]')).not.toBeInViewport();
  await info.attach('injected-defect', {body: 'The unchanged refit contract rejected missing automatic refits.', contentType: 'text/plain'});
});

test('descent, one-step return, breadcrumbs, overview and hidden focus', async ({page}) => {
  await open(page); await clusterContract(page);
  await mouse(page, '#clusterPanel [data-go="D-1"]');
  // D-39: selecting a record no longer moves focus; use the explicit control.
  await mouse(page, '#detailFocus');
  for (const id of ['D-2', 'D-3']) {
    await mouse(page, `[data-node-id="${id}"] > path`);
    await mouse(page, '#detailFocus');
    await expect(page.locator('#focusStatus')).toContainText(`${id} ·`);
    await expect(page.locator('#focusPath [aria-current]')).toContainText(id);
  }
  await mouse(page, '#focusBack');
  await expect(page.locator('#focusStatus')).toContainText('D-2 ·');
  await mouse(page, '[data-node-id="D-3"] > path');
  await mouse(page, '#detailFocus');
  await mouse(page, '#focusPath [data-depth="1"]');
  await expect(page.locator('#focusStatus')).toContainText('D-1 ·');
  await mouse(page, '#focusAll');
  await expect(page.locator('#focusStatus')).toHaveText('All nodes');
  await enter(page, 'D-1');
  await page.locator('#q').fill('NO_SUCH_NODE');
  await expect(page.locator('#focusStatus')).toContainText('Focus hidden');
  await expect(page.locator('#focusPath [aria-current]')).toContainText('D-1');
  await expect(page.locator('[data-node-id]')).toHaveCount(0);
  await mouse(page, '#focusAll');
  await expect(page.locator('#q')).toHaveValue('NO_SUCH_NODE');
  await mouse(page, '#clearFilter');
  await expect(page.locator('#q')).toHaveValue('');
});

test('high degree stays individual with deterministic omission and a route to omitted nodes', async ({page}, info) => {
  await open(page, 'high-degree');
  expect(await page.evaluate(() => window.GY_DATA.nodes.length)).toBe(65);
  await enter(page, 'D-100');
  await expect(page.locator('#focusStatus')).toContainText('1 hop');
  await expect(page.locator('[data-node-id]')).toHaveCount(60);
  await expect(page.locator('rect.node')).toHaveCount(0);
  const expected = ['D-100', 'D-61', ...Array.from({length: 60}, (_, i) => `D-${i+1}`).sort()].slice(0, 60);
  expect(await page.locator('[data-node-id]').evaluateAll(es => es.map(e => e.dataset.nodeId).sort())).toEqual(expected.sort());
  await expect(page.locator('#culling')).toContainText('2 nodes omitted');
  await expect(page.locator('[data-node-id="D-100"]')).toBeInViewport();
  await page.setViewportSize({width:1400,height:500});
  await expect(page.locator('#culling')).toContainText('off-screen at readable zoom');
  await expect(page.locator('#culling')).toContainText('2 nodes omitted');
  await page.screenshot({path: info.outputPath('omitted-and-offscreen.png')});
  await page.setViewportSize({width:1400,height:1000});
  await expect(page.locator('#culling')).not.toContainText('off-screen at readable zoom');
  await mouse(page, '#showOmitted');
  await expect(page.locator('#clusterPanel li')).toHaveCount(2);
  const omitted = await page.locator('#clusterPanel li').first().getAttribute('data-go');
  await mouse(page, page.locator('#clusterPanel li').first());
  await mouse(page, '#detailFocus');
  await expect(page.locator('#focusStatus')).toContainText(`${omitted} ·`);
  await expect(page.locator(`[data-node-id="${omitted}"]`)).toBeInViewport();
  await mouse(page, '#focusBack');
  expect(await page.locator('[data-node-id]').evaluateAll(es => es.map(e => e.dataset.nodeId).sort())).toEqual(expected);
  await page.screenshot({path: info.outputPath('high-degree.png')});
  await mouse(page, '#showOmitted');
  await page.locator('#q').fill('NO_SUCH_NODE');
  await expect(page.locator('#clusterPanel')).not.toHaveClass(/on/);
  await expect(page.locator('#focusStatus')).toContainText('Focus hidden');
  await expect(page.locator('[data-node-id]')).toHaveCount(0);
  await page.locator('#q').fill('');
  await mouse(page, '#focusAll');
  await expect(page.locator('rect.node')).toHaveCount(1);
});

for (const [scenario, size, ledger] of [['overview',1000,'large'], ['focused-star',1000,'large-star'], ['focused-star',3000,'larger-star']]) test(`${size}-node ${scenario} pan and zoom performance`, async ({page, browser}, info) => {
  // Whole-test watchdog includes 24 driver round trips, not a product latency gate.
  test.setTimeout(120000);
  await page.addInitScript(() => {
    window.inputFrames = [];
    const capture = e => {
      if (e.type === 'mousemove' && e.buttons !== 1) return;
      const start = performance.now();
      requestAnimationFrame(() => window.inputFrames.push(performance.now() - start));
    };
    window.addEventListener('wheel', capture, true);
    window.addEventListener('mousemove', capture, true);
  });
  const start = performance.now();
  await open(page, ledger);
  const loadMs = performance.now() - start;
  expect(await page.evaluate(() => window.GY_DATA.nodes.length)).toBe(size);
  const entry = performance.now();
  if (scenario === 'focused-star') {
    await enter(page, 'D-1');
    await expect(page.locator('[data-node-id]')).toHaveCount(60);
    await expect(page.locator('#culling')).toContainText(`${size-60} nodes omitted`);
    const expected = ['D-1', ...Array.from({length:size-1}, (_,i) => `D-${i+2}`).sort().slice(0,59)].sort();
    const selected = await page.locator('[data-node-id]').evaluateAll(es => es.map(e => e.dataset.nodeId).sort());
    expect(selected).toEqual(expected);
    await info.attach('selected-ids.json', {body: JSON.stringify(selected), contentType: 'application/json'});
  }
  const focusEntryMs = scenario === 'focused-star' ? performance.now() - entry : null;
  const timings = [], frameTimings = [];
  const root = page.locator('#svg > g');
  for (let i = 0; i < 24; i++) {
    const box = await page.locator('#svg').evaluate(el => { const r=el.getBoundingClientRect(); return {x:r.x,y:r.y,width:r.width,height:r.height}; });
    const before = await root.getAttribute('transform');
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
    const frameCount = await page.evaluate(() => window.inputFrames.length);
    const tick = performance.now();
    if (i % 2) {
      await page.mouse.wheel(0, i % 4 === 1 ? -80 : 80);
      await expect(root).not.toHaveAttribute('transform', before);
    } else {
      await page.mouse.down();
      await page.mouse.move(box.x + box.width / 2 + 12, box.y + box.height / 2 + 8);
      await page.mouse.up();
      await expect(root).not.toHaveAttribute('transform', before);
    }
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(resolve)));
    const elapsed = performance.now() - tick;
    const frame = await page.evaluate(count => window.inputFrames.slice(count), frameCount);
    expect(frame).toHaveLength(1);
    if (i >= 4) { timings.push(elapsed); frameTimings.push(frame[0]); }
  }
  timings.sort((a,b) => a-b);
  frameTimings.sort((a,b) => a-b);
  const report = {browser: info.project.name, version: browser.version(), os: `${os.platform()} ${os.release()} ${os.arch()}`, cpu: os.cpus()[0].model,
    nodes: size, scenario, focusEntryMs, samples: timings.length, loadMs, medianMs: timings[9], p95Ms: timings[18], maxMs: timings[19], samplesMs: timings,
    frameMedianMs: frameTimings[9], frameP95Ms: frameTimings[18], frameMaxMs: frameTimings[19], frameSamplesMs: frameTimings,
    frameMethod: 'Browser capture-phase input event to next animation frame; excludes driver/IPC/assertion waiting. Not physical display latency.',
    method: 'Mouse command through observed transform change and next animation frame; includes driver/IPC/assertion overhead. Four warmups, 20 samples. Not physical display latency.'};
  await info.attach('performance.json', {body: JSON.stringify(report, null, 2), contentType: 'application/json'});
  // Loose CI regression budgets, not a promise for every device or graph shape.
  expect(loadMs).toBeLessThan(10000);
  // Round-trip p95 is diagnostic only: shared-runner driver/IPC costs vary.
  expect(report.frameP95Ms).toBeLessThan(100);
});

test('detail resizing preserves manual world center and duplicate focus adds no history', async ({page}) => {
  await open(page); await enter(page, 'D-1');
  const count = await page.locator('#focusPath button').count();
  await mouse(page, '[data-node-id="D-1"] > path');
  await expect(page.locator('#focusPath button')).toHaveCount(count);
  await mouse(page, '#zin');
  const world = () => page.locator('#svg').evaluate(svg => {
    const rect = svg.getBoundingClientRect(), m = svg.querySelector(':scope > g').transform.baseVal.consolidate().matrix;
    return {scale:m.a, x:(rect.width/2-m.e)/m.a, y:(rect.height/2-m.f)/m.a};
  });
  const before = await world();
  await mouse(page, '#closeDetail');
  const after = await world();
  for (const k of ['scale','x','y']) expect(after[k]).toBeCloseTo(before[k], 4);
  await expect(page.locator('#focusStatus')).toContainText('D-1 ·');
  await mouse(page, '#focusDetail');
  const reopened = await world();
  for (const k of ['scale','x','y']) expect(reopened[k]).toBeCloseTo(before[k], 4);
  await page.setViewportSize({width:960,height:900});
  await enter(page, 'D-2'); await enter(page, 'D-3');
  await mouse(page, '#focusPath [data-depth="1"]');
  await expect(page.locator('#focusStatus')).toContainText('D-1 ·');
  await mouse(page, '#genealogy');
  await expect(page.locator('#genealogy')).toHaveAttribute('aria-pressed','true');
  await mouse(page, '#focusAll');
  await expect(page.locator('#genealogy')).toHaveAttribute('aria-pressed','true');
});
