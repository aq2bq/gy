import {test, expect} from '@playwright/test';
import {readFileSync, writeFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
import {dir, fixture, mouse, open, enter, scale} from '../helpers.mjs';

async function clusterContract(page) {
  await mouse(page, page.locator('#svg text.nlabel').filter({hasText: 's0 / decision'}));
  await expect(page.locator('#scopeSel'), 'cluster label must filter its member list').toHaveValue('s0');
  await expect(page.locator('[data-kind=decision]')).toHaveAttribute('aria-pressed', 'true');
  expect(await page.locator('#recordList tbody tr').count()).toBe(175);
  expect(new Set(await page.locator('#recordList td[data-column=scope]').allTextContents())).toEqual(new Set(['s0']));
  expect(new Set(await page.locator('#recordList td[data-column=type]').allTextContents())).toEqual(new Set(['decision']));
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
  await expect(clusterContract(page)).rejects.toThrow(/cluster label must filter/);
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
  await mouse(page, '#recordList [data-record="D-1"]');
  // D-39: selecting a record no longer moves focus; use the explicit control.
  await mouse(page, '#applyHop');
  for (const id of ['D-2', 'D-3']) {
    await mouse(page, `[data-node-id="${id}"] > path`);
    await mouse(page, '#applyHop');
    await expect(page.locator('#focusStatus')).toContainText(`${id} ·`);
    await expect(page.locator('#focusPath [aria-current]')).toContainText(id);
  }
  await mouse(page, '#focusBack');
  await expect(page.locator('#focusStatus')).toContainText('D-2 ·');
  await mouse(page, '[data-node-id="D-3"] > path');
  await mouse(page, '#applyHop');
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
  const originalCulling = await page.locator('#culling').textContent();
  const countOffscreen = text => Number(text.match(/(\d+) off-screen/)?.[1] || 0);
  await page.setViewportSize({width:1400,height:500});
  await expect(page.locator('#culling')).toContainText('off-screen at readable zoom');
  await expect(page.locator('#culling')).toContainText('2 nodes omitted');
  await expect.poll(async()=>countOffscreen(await page.locator('#culling').textContent())).toBeGreaterThan(countOffscreen(originalCulling));
  await page.screenshot({path: info.outputPath('omitted-and-offscreen.png')});
  await page.setViewportSize({width:1400,height:1000});
  // D-40 / AC-27: the table reserves height; restoring the viewport restores its measured count.
  await expect(page.locator('#culling')).toHaveText(originalCulling);
  await mouse(page, '#showOmitted');
  await expect(page.locator('#recordList tbody tr')).toHaveCount(65);
  const omitted = await page.locator('#recordList a:focus').getAttribute('data-record');
  expect(expected).not.toContain(omitted);
  const neighbors = await page.evaluate(() => window.GY_DATA.edges.flatMap(e => e.source === 'D-100' ? [e.target] : e.target === 'D-100' ? [e.source] : []));
  expect(neighbors).toContain(omitted);
  await mouse(page, page.locator('#recordList a:focus'));
  await mouse(page, '#applyHop');
  await expect(page.locator('#focusStatus')).toContainText(`${omitted} ·`);
  await expect(page.locator(`[data-node-id="${omitted}"]`)).toBeInViewport();
  await mouse(page, '#focusBack');
  expect(await page.locator('[data-node-id]').evaluateAll(es => es.map(e => e.dataset.nodeId).sort())).toEqual(expected);
  await page.screenshot({path: info.outputPath('high-degree.png')});
  await mouse(page, '#showOmitted');
  await page.locator('#q').fill('NO_SUCH_NODE');
  await expect(page.locator('#recordList tbody tr')).toHaveCount(0);
  await expect(page.locator('#focusStatus')).toContainText('Focus hidden');
  await expect(page.locator('[data-node-id]')).toHaveCount(0);
  await page.locator('#q').fill('');
  await mouse(page, '#focusAll');
  await expect(page.locator('rect.node')).toHaveCount(1);
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
