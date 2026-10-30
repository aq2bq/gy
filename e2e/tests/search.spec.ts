import { expect, test } from '@playwright/test';
import { writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

const palette = (page: import('@playwright/test').Page) => page.getByTestId('palette');

test('/ opens the search and Enter opens the hit', async ({ page, request }) => {
  const needs = (await (await request.get(`${gy.url}api/list?kind=Need`)).json()).rows;
  const hit = needs[0];

  await page.goto(gy.url);
  await page.keyboard.press('/');
  await expect(palette(page)).toBeVisible();

  // The palette asks /api/search as it is typed.
  await palette(page).getByRole('textbox').fill(hit.id);
  await expect(palette(page).getByRole('option').first()).toContainText(hit.id);
  await page.keyboard.press('Enter');
  await expect(page.getByTestId('main').getByRole('heading', { level: 1 })).toHaveText(hit.title);
});

test('a title part finds the node too', async ({ page, request }) => {
  const answer = await (await request.get(`${gy.url}api/search?q=second`)).json();
  const first = answer.hits[0];

  await page.goto(gy.url);
  await page.keyboard.press('/');
  await palette(page).getByRole('textbox').fill('second');
  await expect(palette(page).getByRole('option')).toHaveCount(answer.hits.length);
  await expect(palette(page).getByRole('option').first()).toContainText(first.title);

  await page.keyboard.press('Enter');
  await expect(page.getByTestId('main').getByRole('heading', { level: 1 })).toHaveText(first.title);
});

test('a word only in the body finds the node', async ({ page, request }) => {
  const needs = (await (await request.get(`${gy.url}api/list?kind=Need`)).json()).rows;
  const hit = needs[0];
  const file = join(gy.dir, 'body.md');
  writeFileSync(file, 'the body carries zygomorphic\n');

  const absent = await (await request.get(`${gy.url}api/search?q=zygomorphic`)).json();
  expect(absent.hits).toHaveLength(0);

  gy.gy(['edit', hit.id, '--reason', 'e2e body', '--body-file', file]);

  // The API reads the body, and the palette shows the hit.
  const found = await (await request.get(`${gy.url}api/search?q=zygomorphic`)).json();
  expect(found.hits.map((row: { id: string }) => row.id)).toContain(hit.id);

  await page.goto(gy.url);
  await page.keyboard.press('/');
  await palette(page).getByRole('textbox').fill('zygomorphic');
  await expect(palette(page).getByRole('option').first()).toContainText(hit.id);
});

test('Meta+K and Control+K open, Escape closes', async ({ page }) => {
  await page.goto(gy.url);

  await page.keyboard.press('Meta+k');
  await expect(palette(page)).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(palette(page)).toBeHidden();

  await page.keyboard.press('Control+k');
  await expect(palette(page)).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(palette(page)).toBeHidden();
});

test('/ inside the input stays a character', async ({ page }) => {
  await page.goto(gy.url);
  await page.keyboard.press('/');
  const input = palette(page).getByRole('textbox');
  await expect(input).toBeFocused();

  await input.pressSequentially('/list');
  await expect(input).toHaveValue('/list');
  // The panel stays open: the shortcut only fires outside the input.
  await expect(palette(page)).toBeVisible();
});

test('an Enter that confirms an IME does not open the hit', async ({ page }) => {
  await page.goto(gy.url);
  await page.keyboard.press('/');
  const input = palette(page).getByRole('textbox');
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
  await expect(page.getByTestId('main').getByRole('heading', { level: 1 })).toBeVisible();
});