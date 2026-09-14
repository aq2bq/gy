import {test, expect} from '@playwright/test';
import os from 'node:os';
import {open, enter} from '../helpers.mjs';

// One worker runs the performance samples sequentially, including in local runs.
test.describe.configure({mode:'serial'});

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

