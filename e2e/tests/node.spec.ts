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
const boxes = (path: { up: { nodes: { id: string }[] }[]; down: { nodes: { id: string }[] }[] }) =>
  new Set([...path.up, ...path.down].flatMap(stage => stage.nodes.map(node => node.id))).size;

async function firstNeed(request: import('@playwright/test').APIRequestContext) {
  const rows = (await (await request.get(`${gy.url}api/list?kind=Need`)).json()).rows;
  return rows.find((row: { status?: string }) => row.status === 'open') || rows[0];
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
