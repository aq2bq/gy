import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the three eyes match /api/now', async ({ page, request }) => {
  const answer = await (await request.get(`${gy.url}api/now`)).json();
  await page.goto(gy.url);

  // The three column numbers, then the same three in the sidebar.
  const counts = [
    String(answer.waiting.length),
    String(answer.ready.length),
    String(answer.resume.in_progress.length),
  ];
  await expect(page.locator('.eye .head .n')).toHaveText(counts);
  await expect(page.locator('#eyes3 b')).toHaveText(counts);

  // The sky and the two-column pulse are on the page.
  await expect(page.locator('.sky canvas')).toBeVisible();
  await expect(page.locator('.pl2 .pi')).toHaveCount(answer.recent.length);
});

test('the language switch redraws without a reload', async ({ page }) => {
  await page.goto(gy.url);
  const label = page.locator('.eye .head .l').first();
  const english = await label.innerText();

  // A reload would drop this mark; the switch must keep it.
  await page.evaluate(() => {
    (window as unknown as { kept?: boolean }).kept = true;
  });
  await page.locator('#lang button[data-l="ja"]').click();

  await expect(label).not.toHaveText(english);
  expect(await page.evaluate(() => (window as unknown as { kept?: boolean }).kept)).toBe(true);
});

test('the wordmark returns to the now page from a list', async ({ page }) => {
  await page.goto(`${gy.url}#/list/Need`);
  await page.locator('.wordmark').click();

  await expect(page).toHaveURL(/#\/$/);
  await expect(page.locator('.eyes')).toBeVisible();
});