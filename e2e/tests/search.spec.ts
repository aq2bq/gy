import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('/ opens the search and Enter opens the hit', async ({ page, request }) => {
  const needs = (await (await request.get(`${gy.url}api/list?kind=Need`)).json()).rows;
  const hit = needs[0];

  await page.goto(gy.url);
  await page.keyboard.press('/');
  await expect(page.locator('#pal')).toHaveClass(/on/);

  // The palette asks /api/search as it is typed.
  await page.locator('#palq').fill(hit.id);
  await expect(page.locator('#palres .r').first()).toContainText(hit.id);
  await page.keyboard.press('Enter');
  await expect(page.locator('.nh h1')).toHaveText(hit.title);
});

test('a title part finds the node too', async ({ page, request }) => {
  const answer = await (await request.get(`${gy.url}api/search?q=second`)).json();
  const first = answer.hits[0];

  await page.goto(gy.url);
  await page.keyboard.press('/');
  await page.locator('#palq').fill('second');
  await expect(page.locator('#palres .r')).toHaveCount(answer.hits.length);
  await expect(page.locator('#palres .r').first()).toContainText(first.title);

  await page.keyboard.press('Enter');
  await expect(page.locator('.nh h1')).toHaveText(first.title);
});

test('Meta+K and Control+K open, Escape closes', async ({ page }) => {
  await page.goto(gy.url);

  await page.keyboard.press('Meta+k');
  await expect(page.locator('#pal')).toHaveClass(/on/);
  await page.keyboard.press('Escape');
  await expect(page.locator('#pal')).not.toHaveClass(/on/);

  await page.keyboard.press('Control+k');
  await expect(page.locator('#pal')).toHaveClass(/on/);
  await page.keyboard.press('Escape');
  await expect(page.locator('#pal')).not.toHaveClass(/on/);
});

test('/ inside the input stays a character', async ({ page }) => {
  await page.goto(gy.url);
  await page.keyboard.press('/');
  await expect(page.locator('#palq')).toBeFocused();

  await page.locator('#palq').pressSequentially('/list');
  await expect(page.locator('#palq')).toHaveValue('/list');
  // The panel stays open: the shortcut only fires outside the input.
  await expect(page.locator('#pal')).toHaveClass(/on/);
});

test('an Enter that confirms an IME does not open the hit', async ({ page }) => {
  await page.goto(gy.url);
  await page.keyboard.press('/');
  const input = page.locator('#palq');
  await input.click();
  await page.keyboard.insertText('きろく');
  const before = page.url();

  // The confirming Enter arrives while the IME is composing: not our key.
  await input.evaluate(el => {
    el.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', isComposing: true, bubbles: true, cancelable: true }));
  });
  expect(page.url()).toBe(before);

  // A plain Enter after typing still opens the hit.
  await input.fill('second');
  await page.keyboard.press('Enter');
  await expect(page.locator('.nh h1')).toBeVisible();
});
