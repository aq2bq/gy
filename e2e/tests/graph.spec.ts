import { expect, test, type Page } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

/// Where a world point sits on the canvas right now.
const screen = (page: Page, x: number, y: number) =>
  page.evaluate(
    ({ x, y }) => (window as unknown as { GyGraph: { worldToScreen: (x: number, y: number) => { x: number; y: number } } }).GyGraph.worldToScreen(x, y),
    { x, y },
  );

/// The camera's three numbers, as one comparable string.
const camAt = (page: Page) =>
  page.evaluate(() => {
    const cam = (window as unknown as { GyGraph: { cam: { k: number; x: number; y: number } } }).GyGraph.cam;
    return `${cam.k},${cam.x},${cam.y}`;
  });

/// Wait for the camera to stop moving, not for a fixed time: the next click is
/// aimed at a screen point, so it is only right once the camera has settled
/// (n-fe70). `from` is the value before the ease: a camera that has left it is
/// an ease in flight, and two equal samples 90 ms apart mean it stopped. A
/// camera still on `from` may be an ease that has not started yet, so the same
/// value must hold for a window longer than the ease (8 × 90 ms > FIT_MS 620).
async function settleCamera(page: Page, from: string) {
  let last = '';
  let same = 0;
  let moved = false;
  await expect
    .poll(async () => {
      const now = await camAt(page);
      same = now === last ? same + 1 : 0;
      if (now !== from) moved = true;
      last = now;
      return moved ? same >= 2 : same >= 8;
    }, { message: 'the camera never settled', timeout: 10000, intervals: [90] })
    .toBe(true);
}

test('the graph goes from a bubble to a node to a page', async ({ page, request }) => {
  const graph = await (await request.get(`${gy.url}api/graph`)).json();
  await page.goto(`${gy.url}#/graph`);
  const canvas = page.locator('canvas#g');
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('the canvas has no box');

  // The page fits every node on its first draw: the crumb is only drawn once
// the graph has loaded, and the camera is only still once that ease ends. A
// click before then is aimed at where a bubble used to be.
  await expect(page.locator('#gcrumb')).not.toBeEmpty();
  await settleCamera(page, await camAt(page));

  const bubble = graph.bubbles[0];
  const centre = await screen(page, bubble.x, bubble.y);
  const beforeBubble = await camAt(page);
  await page.mouse.click(box.x + centre.x, box.y + centre.y);
  await settleCamera(page, beforeBubble);

  // A click on a node opens the panel with its words.
  const node = graph.nodes.find((item: { scope: string }) => item.scope === bubble.scope);
  const at = await screen(page, node.x, node.y);
  const beforeNode = await camAt(page);
  await page.mouse.click(box.x + at.x, box.y + at.y);
  const panel = page.locator('.gpanel');
  await expect(panel).toHaveClass(/on/);
  await expect(panel.locator('.tt')).not.toBeEmpty();

  // A double click on the same node opens its page.
  await settleCamera(page, beforeNode);
  const again = await screen(page, node.x, node.y);
  await page.mouse.dblclick(box.x + again.x, box.y + again.y);
  await expect(page).toHaveURL(new RegExp(`#/n/${node.id}$`));
});