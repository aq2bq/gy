import { expect, test, type APIRequestContext, type Locator } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

/// The name a box on the map shows for an edge's peer.
const named = (edge: { alias?: string; to: string }) => edge.alias || edge.to;

/// The decision the node scenes open: it has edges, so its map has peers.
const closes = async (request: APIRequestContext) => {
  const rows = (await (await request.get(`${gy.url}api/list?kind=Decision`)).json()).rows;
  return rows.find((row: { title: string }) => row.title.startsWith('closes'));
};

/// The camera the map's svg carries, as the one place e2e reads it (n-9ca9).
const camera = async (map: Locator) => ({
  k: Number(await map.getAttribute('data-k')),
  x: Number(await map.getAttribute('data-x')),
  y: Number(await map.getAttribute('data-y')),
});

/// The middle of the map, where the pointer goes for a wheel or a drag.
const middle = async (map: Locator) => {
  const box = await map.boundingBox();
  if (!box) throw new Error('the map has no box');
  return { x: box.x + box.width / 2, y: box.y + box.height / 2 };
};

test('the decision page matches /api/node', async ({ page, request }) => {
  const decisions = (await (await request.get(`${gy.url}api/list?kind=Decision`)).json()).rows;
  const decision = decisions.find((row: { title: string }) => row.title.startsWith('closes'));
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  const words = await (await request.get(`${gy.url}assets/i18n.json`)).json();

  await page.goto(`${gy.url}#/n/${decision.id}`);
  const main = page.getByTestId('main');
  await expect(main.getByRole('heading', { level: 1 })).toHaveText(node.title);
  await expect(main.getByText(node.data.Decision.scope.text)).toBeVisible();
  await expect(main.getByRole('list', { name: /History/ }).getByRole('listitem')).toHaveCount(node.history.length);
  // Every node of the neighbourhood is a box, the focus included (n-...).
  await expect(main.locator('svg rect')).toHaveCount(node.neighborhood.nodes.length);

  // The body is open from the start, with no fold to open (n-d9a6).
  await expect(main.locator('summary')).toHaveCount(0);
  await expect(main.getByText(words.en.bodyTitle, { exact: true })).toBeVisible();
  await expect(main.locator('pre')).not.toBeEmpty();

  // A box opens the page of the node it points at.
  const peer = node.edges[0];
  await main.locator('svg a').filter({ hasText: named(peer) }).click();
  await expect(main.getByRole('heading', { level: 1 })).not.toHaveText(node.title);
});

test('the decision names the question it closed', async ({ page, request }) => {
  const decisions = (await (await request.get(`${gy.url}api/list?kind=Decision`)).json()).rows;
  const decision = decisions.find((row: { title: string }) => row.title.startsWith('closes'));
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  const edge = node.edges.find((item: { name: string }) => item.name === 'closed-by');
  const question = await (await request.get(`${gy.url}api/node/${edge.to}`)).json();
  const main = page.getByTestId('main');

  await page.goto(`${gy.url}#/n/${decision.id}`);
  const box = main.locator('svg a').filter({ hasText: named(edge) });
  await expect(box).toHaveCount(1);
  // The relation reads as "closed" / "閉じた論点" from this side (the label
  // sits beside the box, not inside it).
  const labels = await main
    .locator('svg text')
    .evaluateAll(nodes => nodes.map(node => node.textContent));
  expect(labels.join(' ')).toMatch(/closed|閉じた/);

  // The box opens the question's page.
  await box.click();
  await expect(main.getByRole('heading', { level: 1 })).toHaveText(question.title);
  await expect(main.getByRole('list', { name: /History/ }).getByRole('listitem')).toHaveCount(question.history.length);
});

test('the map rings two hops deep and marks the click it came from', async ({ page }) => {
  let first = '', middle = '';
  const chain = await start({
    build: gy => {
      const measure = JSON.parse(gy(['--scope', 'a', 'criterion', 'add', 'the measure'])).id as string;
      const need = (title: string) => JSON.parse(gy(['--scope', 'a', 'need', 'add', title, '--targets', measure])).id as string;
      first = need('the first need');
      middle = need('the middle need');
      const far = need('the far need');
      gy(['--scope', 'a', 'link', first, 'depends-on', middle]);
      gy(['--scope', 'a', 'link', middle, 'depends-on', far]);
    },
  });
  try {
    const main = page.getByTestId('main');
    const answer = await (await page.request.get(`${chain.url}api/node/${first}`)).json();
    const far = answer.neighborhood.nodes.filter((node: { hop: number }) => node.hop === 2);

    // Every node of the two rings is a box, the far ring marked apart.
    await page.goto(`${chain.url}#/n/${first}`);
    await expect(main.locator('svg rect')).toHaveCount(answer.neighborhood.nodes.length);
    await expect(main.locator('svg rect[data-hop="2"]')).toHaveCount(far.length);
    expect(far.length).toBeGreaterThan(0);
    const labels = await main.locator('svg text').evaluateAll(nodes => nodes.map(node => node.textContent));
    expect(labels.join(' ')).toMatch(/depends/);

    // Opening a box keeps the step it came from: a back chip names it and its
    // own box wears the "from" mark.
    await main.locator('svg a').filter({ hasText: middle }).click();
    await expect(main.getByRole('heading', { level: 1 })).toHaveText('the middle need');
    await expect(main.locator('h3 a')).toHaveText(new RegExp(first));
    await expect(main.locator('svg rect[data-from]')).toHaveCount(1);
  } finally {
    await chain.stop();
  }
});

test('the map zooms with the wheel and the keys, and stops at the limits', async ({ page, request }) => {
  const decision = await closes(request);
  await page.goto(`${gy.url}#/n/${decision.id}`);
  const map = page.getByTestId('main').getByTestId('ego');
  await expect(map).toHaveAttribute('data-k', '1');
  const at = await middle(map);

  // The wheel zooms about the pointer.
  await page.mouse.move(at.x, at.y);
  await page.mouse.wheel(0, -240);
  await expect.poll(async () => (await camera(map)).k).toBeGreaterThan(1);

  // `+` zooms a step further; `-` a step back.
  const afterWheel = (await camera(map)).k;
  await page.keyboard.press('+');
  await expect.poll(async () => (await camera(map)).k).toBeGreaterThan(afterWheel);
  const afterKey = (await camera(map)).k;
  await page.keyboard.press('-');
  await expect.poll(async () => (await camera(map)).k).toBeLessThan(afterKey);

  // Neither direction goes past the graph page's own limits.
  for (let step = 0; step < 20; step++) await page.keyboard.press('+');
  await expect.poll(async () => (await camera(map)).k).toBe(12);
  for (let step = 0; step < 40; step++) await page.keyboard.press('-');
  await expect.poll(async () => (await camera(map)).k).toBe(0.15);
});

test('the map pans with a drag and the drag opens no page', async ({ page, request }) => {
  const decision = await closes(request);
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  await page.goto(`${gy.url}#/n/${decision.id}`);
  const main = page.getByTestId('main');
  const map = main.getByTestId('ego');
  const at = await middle(map);
  const before = await camera(map);

  await page.mouse.move(at.x, at.y);
  await page.mouse.down();
  await page.mouse.move(at.x + 80, at.y + 50, { steps: 8 });
  await page.mouse.up();

  await expect.poll(async () => (await camera(map)).x).not.toBe(before.x);
  await expect.poll(async () => (await camera(map)).y).not.toBe(before.y);
  // The drag is not a press on a box: the page stays where it was.
  await expect(main.getByRole('heading', { level: 1 })).toHaveText(node.title);
});

test('a drag begun on a box pans without opening its page', async ({ page, request }) => {
  const decision = await closes(request);
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  const peer = node.edges[0];
  await page.goto(`${gy.url}#/n/${decision.id}`);
  const main = page.getByTestId('main');
  const map = main.getByTestId('ego');
  const box = main.locator('svg a').filter({ hasText: named(peer) }).first();
  const start = await box.boundingBox();
  if (!start) throw new Error('the peer has no box');
  const at = { x: start.x + start.width / 2, y: start.y + start.height / 2 };
  const before = await camera(map);

  await page.mouse.move(at.x, at.y);
  await page.mouse.down();
  await page.mouse.move(at.x + 70, at.y + 40, { steps: 8 });
  await page.mouse.up();

  await expect.poll(async () => (await camera(map)).x).not.toBe(before.x);
  await expect.poll(async () => (await camera(map)).y).not.toBe(before.y);
  // The press began on the anchor and the pan carried it along; it is not a click.
  await expect(main.getByRole('heading', { level: 1 })).toHaveText(node.title);
});

test('a box answers a press on its text', async ({ page, request }) => {
  const decision = await closes(request);
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  const peer = node.edges[0];
  await page.goto(`${gy.url}#/n/${decision.id}`);
  const main = page.getByTestId('main');
  const text = main.locator('svg a text').filter({ hasText: named(peer) }).first();
  await expect(text).toBeVisible();

  await text.click();
  await expect(main.getByRole('heading', { level: 1 })).not.toHaveText(node.title);
});

test('the camera is kept on a redraw and reset on another node', async ({ page, request }) => {
  const decision = await closes(request);
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  const peer = node.edges[0];
  await page.goto(`${gy.url}#/n/${decision.id}`);
  const map = page.getByTestId('main').getByTestId('ego');
  const at = await middle(map);
  await page.mouse.move(at.x, at.y);
  await page.mouse.wheel(0, -240);
  await expect.poll(async () => (await camera(map)).k).toBeGreaterThan(1);
  const zoomed = await camera(map);

  // A language switch redraws the same node: the camera holds.
  await page.getByTestId('sidebar').getByRole('button', { name: '日本語' }).click();
  await expect(map).toHaveAttribute('data-k', String(zoomed.k));
  await expect(map).toHaveAttribute('data-x', String(zoomed.x));

  // Another node opens at the whole figure.
  await page.evaluate(id => { location.hash = `#/n/${id}`; }, peer.to);
  await expect.poll(async () => (await camera(map)).k).toBe(1);
  await expect.poll(async () => (await camera(map)).x).toBe(0);
});

test('a zoom from a scrolled page keeps the page where it was', async ({ page, request }) => {
  const decision = await closes(request);
  // A short window, so the page scrolls and the map can still be seen.
  await page.setViewportSize({ width: 1280, height: 420 });
  await page.goto(`${gy.url}#/n/${decision.id}`);
  const map = page.getByTestId('main').getByTestId('ego');
  await expect(map).toBeVisible();
  await page.evaluate(() => window.scrollTo(0, 220));
  const box = await map.boundingBox();
  if (!box) throw new Error('the map has no box');
  // The point sits in the part of the map the short window shows.
  const top = Math.max(box.y, 0);
  const bottom = Math.min(box.y + box.height, 420);
  expect(bottom).toBeGreaterThan(top);
  const at = { x: box.x + box.width / 2, y: (top + bottom) / 2 };
  const before = await page.evaluate(() => window.scrollY);
  expect(before).toBeGreaterThan(0);

  await page.mouse.move(at.x, at.y);
  await page.mouse.wheel(0, -240);
  await expect.poll(async () => (await camera(map)).k).toBeGreaterThan(1);

  // The zoom does not jump the page back to its top.
  expect(await page.evaluate(() => window.scrollY)).toBe(before);
});

test('a node opened again after another page starts at the top', async ({ page, request }) => {
  const decision = await closes(request);
  await page.setViewportSize({ width: 1280, height: 420 });
  await page.goto(`${gy.url}#/n/${decision.id}`);
  const map = page.getByTestId('main').getByTestId('ego');
  await expect(map).toBeVisible();
  await page.evaluate(() => window.scrollTo(0, 220));
  expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(0);

  // Another page carries no mark of the node that was shown.
  await page.evaluate(() => { location.hash = '#/list/Need'; });
  await expect(page.getByTestId('main').getByRole('heading', { level: 1 })).toBeVisible();

  await page.evaluate(id => { location.hash = `#/n/${id}`; }, decision.id);
  await expect(map).toBeVisible();
  expect(await page.evaluate(() => window.scrollY)).toBe(0);
});
