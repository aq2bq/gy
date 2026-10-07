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

/// One thing the frame decided to paint, and one it decided not to.
type Op = { role?: string; value?: string; x: number; y: number; r?: number; alpha?: number; stroke?: string; lineWidth?: number };
type Frame = {
  bubbles: { scope: string; circle: Op; text: Op }[];
  cross: { from: string; to: string }[];
  edges: { from: string; to: string }[];
  nodes: { id: string; circle: Op; texts: Op[] }[];
  wanted: string[];
};

/// The display list the last frame made: what the page decided to draw, and the
/// labels it still wants. The canvas holds only pixels, so this is the reading
/// (d-8b47).
const drawn = (page: Page) =>
  page.evaluate(() => (window as unknown as { GyDraw: { lastList: () => Frame } }).GyDraw.lastList());

/// The scale the crumb shows, as a number.
const scale = (page: Page) =>
  page.evaluate(() => {
    const text = document.getElementById('lod')?.textContent ?? '';
    return Number((text.match(/×([0-9.]+)/) ?? [])[1]);
  });

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
  await expect(main(page).getByTestId('gpanel').getByText(shown.title)).toBeVisible();

  // A double click on the same node opens its page.
  await settleCamera(page);
  const again = await screen(page, node.x, node.y);
  await page.mouse.dblclick(box.x + again.x, box.y + again.y);
  await expect(page).toHaveURL(new RegExp(`#/n/${node.id}$`));
});

test('a node shows the shared card on hover, at any scale', async ({ page, request }) => {
  const graph = await (await request.get(`${gy.url}api/graph`)).json();
  const words = await (await request.get(`${gy.url}assets/i18n.json`)).json();
  // A node with a state word and a label, so the card carries all five things.
  const node = graph.nodes.find((item: { state: string | null }) => item.state);
  const labels = await (await request.get(`${gy.url}api/labels?ids=${encodeURIComponent(node.id)}`)).json();
  const label = labels.labels[node.id];

  await page.goto(`${gy.url}#/graph`);
  const canvas = main(page).locator('canvas');
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('the canvas has no box');
  await expect(main(page).getByRole('button', { name: /all scopes/ })).toBeVisible();
  await settleCamera(page);
  const card = main(page).getByTestId('mapcard');
  await expect(card).toHaveCount(0);

  // The widest step — bubbles and dots — shows the card too (ac-f333).
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  for (let step = 0; step < 6; step++) await page.mouse.wheel(0, 600);
  let at = await screen(page, node.x, node.y);
  await page.mouse.move(box.x + at.x, box.y + at.y);
  await expect(card).toBeVisible();
  await expect(card.getByTestId('mapcardId')).toHaveText(node.id);
  await expect(card.getByTestId('mapcardKind')).toHaveClass(new RegExp(`k-${node.kind}`));
  await expect(card.getByTestId('mapcardStatus')).toHaveText(words.en.st[node.kind][node.state]);
  await expect(card.getByTestId('mapcardScope')).toHaveText(node.scope);
  await expect(card.getByTestId('mapcardTitle')).toHaveText(label.title);

  // Off the node and the canvas: the card goes.
  await page.mouse.move(box.x - 40, box.y + box.height / 2);
  await expect(card).toHaveCount(0);

  // Close in (dots, ids, titles): the card stays, and the zoom never changes
  // its text (d-1c00).
  at = await screen(page, node.x, node.y);
  await page.mouse.move(box.x + at.x, box.y + at.y);
  await expect(card).toBeVisible();
  const size = await card.getByTestId('mapcardTitle').evaluate(el => getComputedStyle(el).fontSize);
  for (let step = 0; step < 6; step++) await page.mouse.wheel(0, -240);
  await expect(card).toBeVisible();
  expect(await card.getByTestId('mapcardTitle').evaluate(el => getComputedStyle(el).fontSize)).toBe(size);

  // A click still selects (the right-hand card opens); a double click still
  // opens the node's page.
  at = await screen(page, node.x, node.y);
  await page.mouse.click(box.x + at.x, box.y + at.y);
  await expect(main(page).getByRole('link', { name: /Open/ })).toBeVisible();
  await expect(main(page).getByTestId('gpanel').getByText(label.title)).toBeVisible();
  await settleCamera(page);
  const again = await screen(page, node.x, node.y);
  await page.mouse.dblclick(box.x + again.x, box.y + again.y);
  await expect(page).toHaveURL(new RegExp(`#/n/${node.id}$`));
});

test('the drawing list follows the scale for the edges, the ids and the titles', async ({ page, request }) => {
  const graph = await (await request.get(`${gy.url}api/graph`)).json();
  await page.goto(`${gy.url}#/graph`);
  const canvas = main(page).locator('canvas');
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('the canvas has no box');
  await expect(main(page).getByRole('button', { name: /all scopes/ })).toBeVisible();
  await settleCamera(page);

  // Widest: the dots and the scope names alone — no inner edge, no id, no title.
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  for (let step = 0; step < 8; step++) await page.mouse.wheel(0, 600);
  await expect.poll(() => scale(page)).toBeLessThan(0.9);
  const wide = await drawn(page);
  expect(wide.edges).toHaveLength(0);
  expect(wide.nodes.every(node => node.texts.length === 0)).toBe(true);

  // Close in on a node that has an edge: the ids, the titles, and the edges.
  const node = graph.nodes.find((item: { degree: number }) => item.degree > 0);
  const at = await screen(page, node.x, node.y);
  await page.mouse.move(box.x + at.x, box.y + at.y);
  for (let step = 0; step < 5; step++) await page.mouse.wheel(0, -240);
  await expect.poll(() => scale(page)).toBeGreaterThanOrEqual(3.2);
  const close = await drawn(page);
  expect(close.edges.length).toBeGreaterThan(0);
  expect(close.nodes.length).toBeGreaterThan(0);
  expect(close.nodes.every(item => item.texts.some(text => text.role === 'id'))).toBe(true);
  expect(close.nodes.every(item => item.texts.some(text => text.role === 'title'))).toBe(true);
});

test('the drawing list keeps the selection and its neighbours bright, the rest dim', async ({ page, request }) => {
  const graph = await (await request.get(`${gy.url}api/graph`)).json();
  await page.goto(`${gy.url}#/graph`);
  const canvas = main(page).locator('canvas');
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('the canvas has no box');
  await expect(main(page).getByRole('button', { name: /all scopes/ })).toBeVisible();
  await settleCamera(page);

  // Click a node so it is the selection (the camera fits to it).
  const node = graph.nodes.find((item: { degree: number }) => item.degree > 0);
  const at = await screen(page, node.x, node.y);
  await page.mouse.click(box.x + at.x, box.y + at.y);
  await settleCamera(page);

  // Open wide again, so the dimming of the others is in the list too.
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  for (let step = 0; step < 8; step++) await page.mouse.wheel(0, 600);
  await expect.poll(() => scale(page)).toBeLessThan(0.9);
  const frame = await drawn(page);

  const near = new Set([node.id]);
  for (const [a, b] of [...graph.edges, ...graph.cross]) {
    if (a === node.id) near.add(b);
    if (b === node.id) near.add(a);
  }
  const selected = frame.nodes.find(item => item.id === node.id);
  expect(selected?.circle.stroke).toBe('#ece6d8');
  expect(selected?.circle.alpha).toBe(1);
  expect(frame.nodes.filter(item => near.has(item.id)).every(item => item.circle.alpha === 1)).toBe(true);
  const others = frame.nodes.filter(item => !near.has(item.id));
  expect(others.length).toBeGreaterThan(0);
  expect(others.every(item => item.circle.alpha === 0.22)).toBe(true);
});

test('the drawing list leaves out the nodes that fall off the screen', async ({ page, request }) => {
  const graph = await (await request.get(`${gy.url}api/graph`)).json();
  await page.goto(`${gy.url}#/graph`);
  const canvas = main(page).locator('canvas');
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('the canvas has no box');
  await expect(main(page).getByRole('button', { name: /all scopes/ })).toBeVisible();
  await settleCamera(page);

  // Close in on one node: the far ones leave the canvas.
  const node = graph.nodes[0];
  const at = await screen(page, node.x, node.y);
  await page.mouse.move(box.x + at.x, box.y + at.y);
  for (let step = 0; step < 8; step++) await page.mouse.wheel(0, -240);
  await expect.poll(() => scale(page)).toBeGreaterThanOrEqual(3.2);
  const frame = await drawn(page);
  expect(frame.nodes.length).toBeGreaterThan(0);
  expect(frame.nodes.length).toBeLessThan(graph.nodes.length);
  expect(
    frame.nodes.every(
      item => item.circle.x >= -40 && item.circle.x <= box.width + 40 && item.circle.y >= -40 && item.circle.y <= box.height + 40,
    ),
  ).toBe(true);
});

test('the drawing list asks for the labels it cannot name', async ({ page, request }) => {
  const graph = await (await request.get(`${gy.url}api/graph`)).json();
  // No labels at all: every node the frame names is one it still wants.
  await page.route(/\/api\/labels/, route => route.fulfill({ json: { labels: {} } }));
  await page.goto(`${gy.url}#/graph`);
  const canvas = main(page).locator('canvas');
  await expect(canvas).toBeVisible();
  const box = await canvas.boundingBox();
  if (!box) throw new Error('the canvas has no box');
  await expect(main(page).getByRole('button', { name: /all scopes/ })).toBeVisible();
  await settleCamera(page);

  // Wide: no id is shown, so nothing is wanted.
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  for (let step = 0; step < 8; step++) await page.mouse.wheel(0, 600);
  await expect.poll(() => scale(page)).toBeLessThan(0.9);
  expect((await drawn(page)).wanted).toHaveLength(0);

  // Close in: every node on the canvas is named, and its label is still wanted.
  for (let step = 0; step < 5; step++) await page.mouse.wheel(0, -240);
  await expect.poll(() => scale(page)).toBeGreaterThanOrEqual(1.6);
  const frame = await drawn(page);
  expect(frame.wanted.length).toBeGreaterThan(0);
  expect(new Set(frame.wanted)).toEqual(new Set(frame.nodes.map(item => item.id)));
  expect(frame.nodes.every(item => graph.nodes.some((node: { id: string }) => node.id === item.id))).toBe(true);
});