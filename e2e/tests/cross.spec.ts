import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

/* The crossing screen (r-2d02, d-c302): nothing narrowed is a scope × week
   grid; a narrowed set is the scope × time picture, matched against /api/cross. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

const main = (page: import('@playwright/test').Page) => page.getByTestId('main');

test('the entry is the grid, and a cell narrows that scope and week', async ({ page, request }) => {
  const words = (await (await request.get(`${gy.url}assets/i18n.json`)).json()).en;
  const grid = await (await request.get(`${gy.url}api/cross`)).json();
  expect(grid.capped).toBe(true);

  await page.goto(`${gy.url}?lang=en#/cross`);
  await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(words.crossTitle);
  await expect(main(page).getByText(words.notNarrowed)).toBeVisible();
  const moves = (grid.weeks || []).filter((week: { moves: number }) => week.moves > 0);
  await expect(main(page).getByTestId('heatCell')).toHaveCount(moves.length);

  // A cell asks for that week and scope; the picture is the answer's nodes.
  await main(page).getByTestId('heatCell').first().click();
  await expect(page).toHaveURL(/#\/cross\?.*scope=/);
  const query = new URL(page.url()).hash.split('?')[1];
  const cut = await (await request.get(`${gy.url}api/cross?${query}`)).json();
  await expect(main(page).getByTestId('laneNode')).toHaveCount(cut.nodes.length);
});

test('a word draws its set, and a scope chip narrows it', async ({ page, request }) => {
  const cut = await (await request.get(`${gy.url}api/cross?q=need`)).json();
  expect(cut.capped).toBe(false);

  await page.goto(`${gy.url}?lang=en#/cross`);
  await main(page).getByTestId('crossWord').fill('need');
  await main(page).getByTestId('crossWord').press('Enter');
  await expect(page).toHaveURL(/#\/cross\?q=need/);
  await expect(main(page).getByTestId('laneNode')).toHaveCount(cut.nodes.length);

  // The scope chips wear the ledger's own names; one narrows the set.
  const scopes = [...new Set(cut.nodes.map((node: { scope: string }) => node.scope))] as string[];
  const chip = main(page).getByTestId('scopeChip').filter({ hasText: new RegExp(`^${scopes[0]}`) });
  await chip.click();
  await expect(page).toHaveURL(new RegExp(`scope=${scopes[0]}`));
  const narrowed = await (await request.get(`${gy.url}api/cross?q=need&scope=${encodeURIComponent(scopes[0])}`)).json();
  await expect(main(page).getByTestId('laneNode')).toHaveCount(narrowed.nodes.length);
});

test('a node in the summary opens its detail', async ({ page, request }) => {
  const cut = await (await request.get(`${gy.url}api/cross?q=need`)).json();
  const node = cut.nodes[0];
  const detail = await (await request.get(`${gy.url}api/node/${node.id}`)).json();

  await page.goto(`${gy.url}?lang=en#/cross?q=need`);
  await main(page).getByTestId('crossNode').first().click();
  await expect(page).toHaveURL(new RegExp(`#/n/${node.id}$`));
  await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(detail.title);
});

test('a refused cross keeps the frame and says why', async ({ page }) => {
  const words = (await (await page.request.get(`${gy.url}assets/i18n.json`)).json()).en;
  await page.route('**/api/cross*', route =>
    route.fulfill({ status: 503, contentType: 'application/json', body: JSON.stringify({ reason: 'the cut took too long' }) }),
  );
  await page.goto(`${gy.url}?lang=en#/cross`);
  await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(words.crossTitle);
  await expect(main(page).getByTestId('crossWord')).toBeVisible();
  const failed = main(page).getByTestId('failed');
  await expect(failed).toContainText('the cut took too long');
});
