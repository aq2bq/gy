import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the list rows match /api/list', async ({ page, request }) => {
  const rows = (await (await request.get(`${gy.url}api/list?kind=Question`)).json()).rows;
  const open = rows.filter((row: { status?: string }) => row.status === 'open');

  await page.goto(`${gy.url}#/list/Question`);
  const main = page.getByTestId('main');
  // The page opens on the open rows.
  await expect(main.getByRole('link')).toHaveCount(open.length);

  // The subtitle carries the open count and the total.
  const subtitle = main.getByText(/\d+ open of \d+/);
  await expect(subtitle).toContainText(String(open.length));
  await expect(subtitle).toContainText(String(rows.length));
});

test('the open and all switch changes the rows', async ({ page, request }) => {
  const rows = (await (await request.get(`${gy.url}api/list?kind=Question`)).json()).rows;

  await page.goto(`${gy.url}#/list/Question`);
  const main = page.getByTestId('main');
  const open = await main.getByRole('link').count();
  await main.getByRole('button', { name: 'all' }).click();
  await expect(main.getByRole('link')).toHaveCount(rows.length);
  expect(rows.length).toBeGreaterThan(open);
});