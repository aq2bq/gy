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
  // The page opens on the open rows.
  await expect(page.locator('.row')).toHaveCount(open.length);

  // The subtitle carries the open count and the total.
  const subtitle = await page.locator('.list .sub').innerText();
  expect(subtitle).toContain(String(open.length));
  expect(subtitle).toContain(String(rows.length));
});

test('the open and all switch changes the rows', async ({ page, request }) => {
  const rows = (await (await request.get(`${gy.url}api/list?kind=Question`)).json()).rows;

  await page.goto(`${gy.url}#/list/Question`);
  const open = await page.locator('.row').count();
  await page.locator('.chips button[data-f="all"]').click();
  await expect(page.locator('.row')).toHaveCount(rows.length);
  expect(rows.length).toBeGreaterThan(open);
});
