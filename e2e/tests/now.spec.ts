import { expect, test, type Page } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

const main = (page: Page) => page.getByTestId('main');
const eyes = (page: Page) => main(page).getByRole('region');

test('the three eyes match /api/now', async ({ page, request }) => {
  const answer = await (await request.get(`${gy.url}api/now`)).json();
  await page.goto(gy.url);

  // The three column numbers, then the same three in the sidebar.
  const counts = [
    String(answer.waiting.length),
    String(answer.ready.length),
    String(answer.resume.in_progress.length),
  ];
  // The column's own number, not any number the column happens to show: the
  // resume column also prints the open-question count (n-8f60).
  for (let index = 0; index < counts.length; index++) {
    await expect(eyes(page).nth(index).getByTestId('count')).toHaveText(counts[index]);
    await expect(page.getByTestId('eyes').getByRole('link').nth(index)).toContainText(counts[index]);
  }

  // The sky and the two-column pulse are on the page.
  await expect(main(page).locator('canvas')).toBeVisible();
  await expect(main(page).getByRole('list', { name: /Pulse/ }).getByRole('listitem')).toHaveCount(answer.recent.length);
});

test('the language switch redraws without a reload', async ({ page }) => {
  await page.goto(gy.url);
  const label = eyes(page).first();
  const english = await label.innerText();

  // A reload would drop this mark; the switch must keep it.
  await page.evaluate(() => {
    (window as unknown as { kept?: boolean }).kept = true;
  });
  await page.getByTestId('sidebar').getByRole('button', { name: '日本語' }).click();

  await expect(label).not.toHaveText(english);
  expect(await page.evaluate(() => (window as unknown as { kept?: boolean }).kept)).toBe(true);
});

test('the wordmark returns to the now page from a list', async ({ page }) => {
  await page.goto(`${gy.url}#/list/Need`);
  await page.getByTestId('sidebar').getByRole('link').first().click();

  await expect(page).toHaveURL(/#\/$/);
  await expect(eyes(page)).toHaveCount(3);
});