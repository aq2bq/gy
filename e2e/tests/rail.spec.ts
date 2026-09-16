import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the rail shows the ten newest writes, and only on a wide screen', async ({ page, request }) => {
  const answer = await (await request.get(`${gy.url}api/history?limit=10`)).json();
  await page.setViewportSize({ width: 1600, height: 900 });
  await page.goto(gy.url);

  const rows = page.getByTestId('railBody').getByRole('listitem');
  await expect(rows).toHaveCount(10);
  await expect(rows.first()).toContainText(`· ${answer.rows[0].seq}`);
  await expect(page.getByTestId('rail').getByRole('link', { name: /See all/ })).toHaveCount(1);

  await page.setViewportSize({ width: 1152, height: 720 });
  await expect(page.getByTestId('rail')).toBeHidden();
  await expect(rows).toHaveCount(0);

  // Dragging the window back follows: the rows return.
  await page.setViewportSize({ width: 1600, height: 900 });
  await expect(rows).toHaveCount(10);
});

test('the search lives in the rail when wide and in the top bar when narrow', async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 900 });
  await page.goto(gy.url);
  await expect(page.getByTestId('rail').getByTestId('search')).toHaveCount(1);
  await expect(page.getByTestId('topbar').getByTestId('search')).toHaveCount(0);
  await page.keyboard.press('Control+k');
  await expect(page.getByTestId('palette')).toBeVisible();
  await page.keyboard.press('Escape');

  await page.setViewportSize({ width: 1152, height: 720 });
  await expect(page.getByTestId('topbar').getByTestId('search')).toHaveCount(1);
  await expect(page.getByTestId('rail').getByTestId('search')).toHaveCount(0);
  await page.keyboard.press('Control+k');
  await expect(page.getByTestId('palette')).toBeVisible();
  await page.keyboard.press('Escape');

  // One element moves: it is never doubled or dropped on the way.
  await page.setViewportSize({ width: 1600, height: 900 });
  await expect(page.getByTestId('search')).toHaveCount(1);
  await expect(page.getByTestId('rail').getByTestId('search')).toHaveCount(1);
});

test('the search outlives the panel redraws a scope switch causes', async ({ page, request }) => {
  const errors: Error[] = [];
  page.on('pageerror', error => errors.push(error));

  await page.setViewportSize({ width: 1600, height: 900 });
  await page.goto(gy.url);
  await expect(page.getByTestId('rail').getByTestId('search')).toHaveCount(1);

  // Two switches: the panel is rewritten each time, and the fault only shows
  // on the second one (n-1d12).
  const shell = await (await request.get(`${gy.url}api/shell`)).json();
  const names: string[] = shell.scopes.map((item: { name: string }) => item.name);
  const scopes = page.getByTestId('scopes');
  await scopes.getByRole('button', { name: new RegExp(`^${names[0]}`) }).click();
  await expect(page.getByTestId('search')).toHaveCount(1);
  await expect(page.getByTestId('rail').getByTestId('search')).toHaveCount(1);
  await scopes.getByRole('button', { name: new RegExp(`^${names[1]}`) }).click();
  await expect(page.getByTestId('search')).toHaveCount(1);
  await expect(page.getByTestId('rail').getByTestId('search')).toHaveCount(1);
  await expect(page.getByTestId('rail').getByText(/Live/)).toHaveCount(1);

  expect(errors).toEqual([]);
});