import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

/* Kokopelli's size (2,170 nodes) is why the screen was rebuilt (n-a9f6,
   r-2d02): the first screen draws its answer from one overview, and a set
   past 900 is a grid, not a picture. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start({ large: true });
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the first screen draws a big ledger in three seconds', async ({ page, request }) => {
  const view = await (await request.get(`${gy.url}api/overview`)).json();
  const started = Date.now();
  await page.goto(`${gy.url}?lang=en`);
  await expect(page.getByTestId('main').getByTestId('needCard')).toHaveCount(view.needs.length);
  expect(Date.now() - started).toBeLessThan(3000);
});

test('a set past 900 is a grid, drawn in three seconds', async ({ page }) => {
  const started = Date.now();
  await page.goto(`${gy.url}?lang=en#/cross`);
  await expect(page.getByTestId('main').getByTestId('heatCell').first()).toBeVisible();
  expect(Date.now() - started).toBeLessThan(3000);
});

test('a narrowed set draws its own nodes, not the whole ledger', async ({ page, request }) => {
  const cut = await (await request.get(`${gy.url}api/cross?q=the big`)).json();
  expect(cut.capped).toBe(false);
  await page.goto(`${gy.url}?lang=en#/cross?q=the big`);
  await expect(page.getByTestId('main').getByTestId('laneNode')).toHaveCount(cut.nodes.length);
});
