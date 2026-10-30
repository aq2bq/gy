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

  const found = await (await request.get(`${gy.url}api/search?q=zygomorphic`)).json();
  expect(found.hits.map((row: { id: string }) => row.id)).toContain(hit.id);

  await page.goto(gy.url);
  await page.keyboard.press('/');
  await palette(page).getByRole('textbox').fill('zygomorphic');
  await expect(palette(page).getByRole('option').first()).toContainText(hit.id);
});
