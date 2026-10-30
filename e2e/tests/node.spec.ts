import { expect, test, type Page } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

/* The detail screen (r-2d02, d-e239): left is where the node came from, right
   is what it works on; a box centers its node. The thread comes from the path
   API, the facts and body from the node API, so one can fail alone. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

const main = (page: Page) => page.getByTestId('main');
const boxes = (path: { up: { nodes: unknown[] }[]; down: { nodes: unknown[] }[] }) =>
  [...path.up, ...path.down].reduce((sum, stage) => sum + stage.nodes.length, 0);

async function firstNeed(request: import('@playwright/test').APIRequestContext) {
  const view = await (await request.get(`${gy.url}api/overview`)).json();
  return view.needs[0];
}

test('the thread matches the path, and a box centers its node', async ({ page, request }) => {
  const need = await firstNeed(request);
  const node = await (await request.get(`${gy.url}api/node/${need.id}`)).json();
  const path = await (await request.get(`${gy.url}api/node/${need.id}/path?depth=2`)).json();

  await page.goto(`${gy.url}?lang=en#/n/${need.id}`);
  await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(node.title);
  await expect(main(page).getByTestId('threadCenter')).toHaveCount(1);
  await expect(main(page).getByTestId('threadBox')).toHaveCount(boxes(path));
  await expect(main(page).getByTestId('facts')).toBeVisible();
  await expect(main(page).locator('pre')).not.toBeEmpty();
  await expect(main(page).getByRole('list', { name: /History/ }).getByRole('listitem')).toHaveCount(node.history.length);
  await expect(main(page).getByTestId('crossLinks').getByRole('link').first()).toBeVisible();

  if (boxes(path) > 0) {
    const first = path.up[0]?.nodes[0] || path.down[0]?.nodes[0];
    await main(page).getByTestId('threadBox').first().click();
    await expect(page).toHaveURL(new RegExp(`#/n/${first.id}$`));
    await expect(main(page).getByRole('heading', { level: 1 })).not.toHaveText(node.title);
  }
});

test('three steps asks the path for depth 3', async ({ page, request }) => {
  const need = await firstNeed(request);
  const deep = await (await request.get(`${gy.url}api/node/${need.id}/path?depth=3`)).json();

  await page.goto(`${gy.url}?lang=en#/n/${need.id}`);
  const asked = page.waitForRequest(request => request.url().includes(`/api/node/${need.id}/path?depth=3`));
  await main(page).getByTestId('depth').last().click();
  await asked;
  await expect(main(page).getByTestId('threadBox')).toHaveCount(boxes(deep));
});

test('a refused thread leaves the facts and body in place', async ({ page, request }) => {
  const need = await firstNeed(request);
  const node = await (await request.get(`${gy.url}api/node/${need.id}`)).json();
  await page.route('**/api/node/*/path*', route =>
    route.fulfill({ status: 503, contentType: 'application/json', body: JSON.stringify({ reason: 'the thread took too long' }) }),
  );
  await page.goto(`${gy.url}?lang=en#/n/${need.id}`);
  await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(node.title);
  await expect(main(page).getByTestId('failed')).toContainText('the thread took too long');
  // What does not depend on the thread is still there (ac-0dbb).
  await expect(main(page).locator('pre')).toBeVisible();
  await expect(main(page).getByTestId('facts')).toBeVisible();
});

/// The name a box on the thread shows for an edge's peer.
const named = (edge: { alias?: string; to: string }) => edge.alias || edge.to;

/// The decision these scenes open: it closes a question and serves a criterion,
/// so its thread has a box on each side.
async function closes(request: import('@playwright/test').APIRequestContext) {
  const rows = (await (await request.get(`${gy.url}api/list?kind=Decision`)).json()).rows;
  return rows.find((row: { title: string }) => row.title.startsWith('closes'));
}

test('the decision page matches /api/node', async ({ page, request }) => {
  const decision = await closes(request);
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  const path = await (await request.get(`${gy.url}api/node/${decision.id}/path?depth=2`)).json();
  const words = await (await request.get(`${gy.url}assets/i18n.json`)).json();

  await page.goto(`${gy.url}?lang=en#/n/${decision.id}`);
  const view = main(page);
  await expect(view.getByRole('heading', { level: 1 })).toHaveText(node.title);
  await expect(view.getByText(node.data.Decision.scope.text)).toBeVisible();
  await expect(view.getByRole('list', { name: /History/ }).getByRole('listitem')).toHaveCount(node.history.length);
  // Every node of the thread is a box, the focus included (d-e239).
  await expect(view.getByTestId('threadCenter')).toHaveCount(1);
  await expect(view.getByTestId('threadBox')).toHaveCount(boxes(path));

  // The body is open from the start, with no fold to open (n-d9a6).
  await expect(view.locator('summary')).toHaveCount(0);
  await expect(view.getByText(words.en.bodyTitle, { exact: true })).toBeVisible();
  await expect(view.locator('pre')).not.toBeEmpty();
});

test('the decision names the question it closed', async ({ page, request }) => {
  const decision = await closes(request);
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  const edge = node.edges.find((item: { name: string }) => item.name === 'closed-by');
  const question = await (await request.get(`${gy.url}api/node/${edge.to}`)).json();
  const view = main(page);

  await page.goto(`${gy.url}?lang=en#/n/${decision.id}`);
  const box = view.locator('svg a').filter({ hasText: named(edge) }).first();
  await expect(box).toHaveCount(1);
  // The relation reads as "closed" from this side (the label sits beside the
  // box, not inside it).
  const labels = await view.locator('svg text').evaluateAll(nodes => nodes.map(node => node.textContent));
  expect(labels.join(' ')).toMatch(/closed|閉じた/);

  // The box opens the question's page.
  await box.click();
  await expect(view.getByRole('heading', { level: 1 })).toHaveText(question.title);
  await expect(view.getByRole('list', { name: /History/ }).getByRole('listitem')).toHaveCount(question.history.length);
});

test('a node opened again after another page starts at the top', async ({ page, request }) => {
  const decision = await closes(request);
  await page.setViewportSize({ width: 1280, height: 420 });
  await page.goto(`${gy.url}#/n/${decision.id}`);
  const center = page.getByTestId('main').getByTestId('threadCenter');
  await expect(center).toBeVisible();
  await page.evaluate(() => window.scrollTo(0, 220));
  expect(await page.evaluate(() => window.scrollY)).toBeGreaterThan(0);

  // Another page carries no mark of the node that was shown.
  await page.evaluate(() => { location.hash = '#/list/Need'; });
  await expect(page.getByTestId('main').getByRole('heading', { level: 1 })).toBeVisible();

  await page.evaluate(id => { location.hash = `#/n/${id}`; }, decision.id);
  await expect(center).toBeVisible();
  expect(await page.evaluate(() => window.scrollY)).toBe(0);
});
