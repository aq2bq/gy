import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  // A directory whose ledger has never been written.
  gy = await start({ build: () => {} });
});

test.afterAll(async () => {
  await gy?.stop();
});

test('an unwritten ledger opens, and its first write wakes the page', async ({ page, request }) => {
  const words = await (await request.get(`${gy.url}assets/i18n.json`)).json();
  const praise = [...words.en.praiseWait, ...words.en.praiseResume, ...words.en.praiseQuiet, ...words.en.blockedNext];

  await page.goto(gy.url);
  const eyes = page.getByTestId('main').getByRole('region');
  const note = eyes.nth(0).getByRole('status');
  await expect(note).toHaveText(words.en.firstRun);
  expect(praise).not.toContain(await note.innerText());
  // Only the first eye speaks: the other two stay silent.
  await expect(eyes.nth(1).getByRole('status')).toHaveCount(0);
  await expect(eyes.nth(2).getByRole('status')).toHaveCount(0);

  // One write wakes the page with no reload: the note turns into a praise word.
  gy.gy(['--scope', 'a', 'criterion', 'add', 'the first write']);
  await expect(note).not.toHaveText(words.en.firstRun);
  expect(praise).toContain(await note.innerText());
});