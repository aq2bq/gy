import { expect, test, type Page } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

/// Where a world point sits on the canvas right now. The canvas is drawn, so a
/// node's place is nowhere in the DOM: this is the one handle the page still
/// offers for aiming a click (n-5e43).
const screen = (page: Page, x: number, y: number) =>
  page.evaluate(
    ({ x, y }) => (window as unknown as { GyGraph: { worldToScreen: (x: number, y: number) => { x: number; y: number } } }).GyGraph.worldToScreen(x, y),
    { x, y },
  );

const main = (page: Page) => page.getByTestId('main');

/// Wait for the camera to come to rest, not for a fixed time: the next click is
/// aimed at a screen point, so it is only right once the page says it settled.
/// The page may not have started its fit yet, so a settled window longer than
/// the ease (8 × 90 ms > 620 ms) is what counts when it never left (n-fe70).
async function settleCamera(page: Page) {
  const settled = main(page).locator('[data-settled]');
  let last = '';
  let same = 0;
  let moved = false;
  await expect
    .poll(async () => {
      const now = await settled.getAttribute('data-settled');
      same = now === last ? same + 1 : 0;
      if (now === 'false') moved = true;
      last = now;
      return moved ? now === 'true' : same >= 8;
    }, { message: 'the camera never settled', timeout: 10000, intervals: [90] })
    .toBe(true);
}

test('the graph goes from a bubble to a node to a page', async ({ page, request }) => {
  const graph = await (await request.get(`${gy.url}api/graph`)).json();
  await page.goto(`${gy.url}#/graph`);
  const canvas = main(page).locator('canvas');
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('the canvas has no box');

  // The page fits every node on its first draw: the crumb is only drawn once
// the graph has loaded, and the camera is only still once that ease ends. A
// click before then is aimed at where a bubble used to be.
  await expect(main(page).getByRole('button', { name: /all scopes/ })).toBeVisible();
  await settleCamera(page);

  const bubble = graph.bubbles[0];
  const centre = await screen(page, bubble.x, bubble.y);
  await page.mouse.click(box.x + centre.x, box.y + centre.y);
  await settleCamera(page);

  // A click on a node opens the panel with its words.
  const node = graph.nodes.find((item: { scope: string }) => item.scope === bubble.scope);
  const at = await screen(page, node.x, node.y);
  await page.mouse.click(box.x + at.x, box.y + at.y);
  const shown = await (await request.get(`${gy.url}api/node/${node.id}`)).json();
  await expect(main(page).getByRole('link', { name: /Open/ })).toBeVisible();
  await expect(main(page).getByText(shown.title)).toBeVisible();

  // A double click on the same node opens its page.
  await settleCamera(page);
  const again = await screen(page, node.x, node.y);
  await page.mouse.dblclick(box.x + again.x, box.y + again.y);
  await expect(page).toHaveURL(new RegExp(`#/n/${node.id}$`));
});