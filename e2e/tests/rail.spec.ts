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

  await expect(page.locator('#rail .hi')).toHaveCount(10);
  await expect(page.locator('#rail .hi .when').first()).toContainText(`· ${answer.rows[0].seq}`);
  await expect(page.locator('#rail a[href="#/history"]')).toHaveCount(1);

  await page.setViewportSize({ width: 1152, height: 720 });
  await expect(page.locator('#rail')).toBeHidden();
  await expect(page.locator('#rail .hi')).toHaveCount(0);

  // Dragging the window back follows: the rows return.
  await page.setViewportSize({ width: 1600, height: 900 });
  await expect(page.locator('#rail .hi')).toHaveCount(10);
});