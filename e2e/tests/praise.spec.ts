import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  // Nothing waits on a person and no requirement is filed, so waiting is empty:
  // the one open question has a decider that writes this ledger itself (d-b02d).
  // Two needs wait on it, so none is ready while both are in progress.
  gy = await start({
    build: (gy) => {
      const id = (args: string[]) => JSON.parse(gy(args)).id as string;
      const a = ['--scope', 'a'];
      const criterion = id([...a, 'criterion', 'add', 'measures one']);
      const one = id([...a, 'need', 'add', 'the blocked need', '--targets', criterion]);
      const two = id([...a, 'need', 'add', 'the other blocked need', '--targets', criterion]);
      const question = id([...a, 'question', 'add', 'a question for a writer', '--decider', 'e2e', '--options', 'one', '--options', 'two']);
      gy([...a, 'link', one, 'waits-on', question]);
      gy([...a, 'link', two, 'waits-on', question]);
    },
  });
});

test.afterAll(async () => {
  await gy?.stop();
});

test('a quiet counter is praised, and a jammed next is told plainly', async ({ page, request }) => {
  // The two in_progress counts are different things: needs here, requirements below.
  const view = await (await request.get(`${gy.url}api/now`)).json();
  expect(view.waiting.length).toBe(0);
  expect(view.ready.length).toBe(0);
  expect(view.in_progress.length).toBe(2);
  expect(view.resume.in_progress.length).toBe(0);
  const words = await (await request.get(`${gy.url}assets/i18n.json`)).json();

  await page.goto(gy.url);
  const eyes = page.getByTestId('main').getByRole('region');

  // Waiting is empty: a praise word from the bundle.
  const wait = await eyes.nth(0).getByRole('status').innerText();
  expect(words.en.praiseWait).toContain(wait);

  // Requirements in progress are empty: praised too.
  const resume = await eyes.nth(2).getByRole('status').innerText();
  expect(words.en.praiseResume).toContain(resume);

  // Ready is empty, but 2 needs are in progress: a plain word, not praise.
  await expect(eyes.nth(1).getByRole('status')).toHaveCount(1);
  await expect(eyes.nth(1).getByRole('status')).toHaveText(words.en.blockedNext[0]);
  // The count is worth showing exactly when needs are stuck.
  await expect(eyes.nth(1).getByRole('link')).toContainText('2');
  await expect(eyes.nth(0).getByRole('link')).toHaveCount(0);
  await expect(eyes.nth(2).getByRole('link')).toHaveCount(0);
});