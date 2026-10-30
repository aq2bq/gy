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
  await page.goto(`${gy.url}#/now`);

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

  // The two-column pulse is on the page. (The sky of the graph's nodes is not:
  // it was the all-node placement, which d-159a takes out of serve.)
  await expect(main(page).getByRole('list', { name: /Pulse/ }).getByRole('listitem')).toHaveCount(answer.recent.length);
});

test('the wordmark returns to the progress page from a list', async ({ page }) => {
  await page.goto(`${gy.url}#/list/Need`);
  await page.getByTestId('sidebar').getByRole('link').first().click();

  await expect(page).toHaveURL(/#\/$/);
  await expect(main(page).getByTestId('stats')).toBeVisible();
});
