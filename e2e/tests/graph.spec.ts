import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

/// Where a world point sits on the canvas right now.
const screen = (page: import('@playwright/test').Page, x: number, y: number) =>
  page.evaluate(
    ({ x, y }) => (window as unknown as { GyGraph: { worldToScreen: (x: number, y: number) => { x: number; y: number } } }).GyGraph.worldToScreen(x, y),
    { x, y },
  );

const zoom = (page: import('@playwright/test').Page) =>
  page.evaluate(() => (window as unknown as { GyGraph: { cam: { k: number } } }).GyGraph.cam.k);

test('the graph goes from a bubble to a node to a page', async ({ page, request }) => {
  const graph = await (await request.get(`${gy.url}api/graph`)).json();
  await page.goto(`${gy.url}#/graph`);
  const canvas = page.locator('canvas#g');
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('the canvas has no box');

  const bubble = graph.bubbles[0];
  const centre = await screen(page, bubble.x, bubble.y);
  await page.mouse.click(box.x + centre.x, box.y + centre.y);
  await expect.poll(() => zoom(page)).toBeGreaterThan(0.9);
  await page.waitForTimeout(700);

  // A click on a node opens the panel with its words.
  const node = graph.nodes.find((item: { scope: string }) => item.scope === bubble.scope);
  const at = await screen(page, node.x, node.y);
  await page.mouse.click(box.x + at.x, box.y + at.y);
  const panel = page.locator('.gpanel');
  await expect(panel).toHaveClass(/on/);
  await expect(panel.locator('.tt')).not.toBeEmpty();

  // A double click on the same node opens its page.
  await page.waitForTimeout(400);
  const again = await screen(page, node.x, node.y);
  await page.mouse.dblclick(box.x + again.x, box.y + again.y);
  await expect(page).toHaveURL(new RegExp(`#/n/${node.id}$`));
});
