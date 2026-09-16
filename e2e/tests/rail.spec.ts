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

test('the search lives in the rail when wide and in the top bar when narrow', async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 900 });
  await page.goto(gy.url);
  await expect(page.locator('#rail #searchbtn')).toHaveCount(1);
  await expect(page.locator('.topbar #searchbtn')).toHaveCount(0);
  await page.keyboard.press('Control+k');
  await expect(page.locator('#pal')).toHaveClass(/on/);
  await page.keyboard.press('Escape');

  await page.setViewportSize({ width: 1152, height: 720 });
  await expect(page.locator('.topbar #searchbtn')).toHaveCount(1);
  await expect(page.locator('#rail #searchbtn')).toHaveCount(0);
  await page.keyboard.press('Control+k');
  await expect(page.locator('#pal')).toHaveClass(/on/);
  await page.keyboard.press('Escape');

  // One element moves: it is never doubled or dropped on the way.
  await page.setViewportSize({ width: 1600, height: 900 });
  await expect(page.locator('#searchbtn')).toHaveCount(1);
  await expect(page.locator('#rail #searchbtn')).toHaveCount(1);
});

test('the search outlives the panel redraws a scope switch causes', async ({ page }) => {
  const errors: Error[] = [];
  page.on('pageerror', error => errors.push(error));

  await page.setViewportSize({ width: 1600, height: 900 });
  await page.goto(gy.url);
  await expect(page.locator('#rail #searchbtn')).toHaveCount(1);

  // Two switches: the panel is rewritten each time, and the fault only shows
  // on the second one (n-1d12).
  const scopes = page.locator('#nav button[data-s]:not([data-s="all"])');
  await scopes.nth(0).click();
  await expect(page.locator('#searchbtn')).toHaveCount(1);
  await expect(page.locator('#rail #searchbtn')).toHaveCount(1);
  await scopes.nth(1).click();
  await expect(page.locator('#searchbtn')).toHaveCount(1);
  await expect(page.locator('#rail #searchbtn')).toHaveCount(1);
  await expect(page.locator('#rail .rail-h')).toHaveCount(1);

  expect(errors).toEqual([]);
});