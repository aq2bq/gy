import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the now page matches /api/now', async ({ page, request }) => {
  const answer = await (await request.get(`${gy.url}api/now`)).json();
  await page.goto(gy.url);

  // The heading counts the waits; the first four become cards.
  await expect(page.locator('.hero h1')).toContainText(String(answer.waiting.length));
  await expect(page.locator('.wait')).toHaveCount(Math.min(4, answer.waiting.length));

  // The three sections count what the answer counts.
  const counts = await page.locator('.sec h2 .cnt').allInnerTexts();
  expect(counts).toEqual([
    String(answer.in_progress.length),
    String(answer.open_questions.length),
    String(answer.unmet.length),
  ]);

  // The pulse shows every recent write.
  await expect(page.locator('.pi')).toHaveCount(answer.recent.length);
});

test('the language switch redraws without a reload', async ({ page }) => {
  await page.goto(gy.url);
  const heading = page.locator('.hero h1');
  const english = await heading.innerText();

  // A reload would drop this mark; the switch must keep it.
  await page.evaluate(() => {
    (window as unknown as { kept?: boolean }).kept = true;
  });
  await page.locator('#lang button[data-l="ja"]').click();

  await expect(heading).not.toHaveText(english);
  expect(await page.evaluate(() => (window as unknown as { kept?: boolean }).kept)).toBe(true);
});

test('the wordmark returns to the now page from a list', async ({ page }) => {
  await page.goto(`${gy.url}#/list/Need`);
  await page.locator('.wordmark').click();

  await expect(page).toHaveURL(/#\/$/);
  await expect(page.locator('.hero h1')).toBeVisible();
});
