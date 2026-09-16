import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('a scope wears one badge everywhere, and only a scope wears it', async ({ page }) => {
  await page.goto(`${gy.url}#/list/Need`);
  const row = page.locator('#main .row.wide').first();
  const badge = row.locator('.sb');
  await expect(badge).toHaveCount(1);
  const name = await badge.innerText();
  const listColor = await badge.evaluate(el => getComputedStyle(el).borderColor);

  // The created date keeps its plain cell: one badge per row, for the scope.
  await expect(row.locator('.sc')).toHaveCount(1);
  await expect(row.locator('.sc .sb')).toHaveCount(0);

  // The same node's page wears the same badge in the same colour.
  await row.click();
  await expect(page).toHaveURL(/#\/n\//);
  const meta = page.locator('#main .nh .meta .sb');
  await expect(meta).toHaveText(name);
  expect(await meta.evaluate(el => getComputedStyle(el).borderColor)).toBe(listColor);

  // "all scopes" is not a scope's name, so it stays plain.
  await expect(page.locator('#nav button[data-s="all"] .sb')).toHaveCount(0);
});