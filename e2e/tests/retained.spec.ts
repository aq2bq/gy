import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

// Replacing a visualization must retain the other reading screens and APIs.
let gy: Ledger;
test.beforeAll(async () => { gy = await start(); });
test.afterAll(async () => { await gy?.stop(); });

for (const [route, api] of [
  ['overview', 'overview'],
  ['now', 'now'],
  ['list/Question', 'list?kind=Question'],
  ['history', 'history'],
  ['cross', 'cross'],
]) {
  test(`${route} retains its API, navigation and reading surface`, async ({ page, request }) => {
    const response = await request.get(`${gy.url}api/${api}`);
    expect(response.status()).toBe(200);
    expect(await response.json()).toBeTruthy();
    const errors: string[] = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.goto(`${gy.url}?lang=en#/${route}`);
    await expect(page.getByTestId('sidebar')).toBeVisible();
    await expect(page.getByTestId('main').getByRole('heading').first()).toBeVisible();
    expect(errors).toEqual([]);
  });
}
