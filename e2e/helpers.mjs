import {expect} from '@playwright/test';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
export const dir = fileURLToPath(new URL('./.generated/', import.meta.url));
export const fixture = name => pathToFileURL(resolve(dir, name, 'gy.html')).href;

// Re-resolve the locator and its current box after every redraw/scroll/resize.
// Read the DOM rect: Playwright boundingBox returned different SVG coordinates
// in WebKit during earlier validation. Keep this if browser coverage expands.
// Browser mouse input deliberately exercises hit testing, including label overlays.
export async function mouse(page, selector) {
  const target = typeof selector === 'string' ? page.locator(selector) : selector;
  await target.scrollIntoViewIfNeeded();
  const box = await target.evaluate(el => { const r = el.getBoundingClientRect(); return {x:r.x, y:r.y, width:r.width, height:r.height}; });
  expect(box).not.toBeNull();
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.up();
}
export async function open(page, name = 'large') {
  await page.goto(fixture(name));
  await expect(page.locator('#metaNodes')).not.toBeEmpty();
}
export async function enter(page, id) {
  await mouse(page, '[data-tab="filters"]');
  await page.locator('#hopFrom').fill(id);
  await mouse(page, '#applyHop');
  await expect(page.locator('#focusStatus')).toContainText(`Focus: ${id} ·`);
}
export async function scale(page) {
  return page.locator('#svg > g').evaluate(el => el.transform.baseVal.consolidate().matrix.a);
}
