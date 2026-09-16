import { expect, test } from '@playwright/test';
import { basename } from 'node:path';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the tab names the ledger directory', async ({ page }) => {
  await page.goto(gy.url);
  await expect(page).toHaveTitle(`gy - ${basename(gy.dir)}`);
});