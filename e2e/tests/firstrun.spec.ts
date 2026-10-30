import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

/* A ledger with no write opens, and the first write wakes the page with no
   reload (n-3e6b). The first screen is the progress board. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start({ build: () => {} });
});

test.afterAll(async () => {
  await gy?.stop();
});

test('an unwritten ledger opens, and its first write wakes the page', async ({ page, request }) => {
  const words = (await (await request.get(`${gy.url}assets/i18n.json`)).json()).en;
  await page.goto(`${gy.url}?lang=en`);
  const main = page.getByTestId('main');
  await expect(main.getByRole('heading', { level: 1 })).toHaveText(words.overviewTitle);
  await expect(main.getByTestId('needCard')).toHaveCount(0);
  await expect(main.getByTestId('stat').first().getByTestId('statValue')).toHaveText('0');

  // A reload would drop this mark; the write must bring the card without one.
  await page.evaluate(() => {
    (window as unknown as { kept?: boolean }).kept = true;
  });
  const criterion = JSON.parse(gy.gy(['--scope', 'a', 'criterion', 'add', 'the first measure'])).id as string;
  gy.gy(['--scope', 'a', 'need', 'add', 'the first need', '--targets', criterion]);

  await expect(main.getByTestId('needCard')).toHaveCount(1);
  await expect(main.getByTestId('needCard').first()).toContainText('the first need');
  expect(await page.evaluate(() => (window as unknown as { kept?: boolean }).kept)).toBe(true);
});
