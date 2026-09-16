import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start({ large: true });
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the now page and a long list draw in two seconds', async ({ page }) => {
  const started = Date.now();
  await page.goto(gy.url);
  await expect(page.getByTestId('main').getByRole('region')).toHaveCount(3);
  expect(Date.now() - started).toBeLessThan(2000);

  const listed = Date.now();
  await page.evaluate(() => {
    location.hash = '#/list/Decision';
  });
  await expect(page.getByTestId('main').getByRole('link')).toHaveCount(60);
  expect(Date.now() - listed).toBeLessThan(2000);
});

test('the search stops at ten hits', async ({ page }) => {
  await page.goto(gy.url);
  await page.keyboard.press('/');
  await page.getByTestId('palette').getByRole('textbox').fill('decision');
  await expect(page.getByTestId('palette').getByRole('option')).toHaveCount(10);
});