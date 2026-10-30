import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

/* The first screen (r-2d02, d-e239): the frame before the answer, and the
   answer matched against /api/overview. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

const main = (page: import('@playwright/test').Page) => page.getByTestId('main');

test('the frame is drawn before the answer, and the board matches /api/overview', async ({ page, request }) => {
  const words = (await (await request.get(`${gy.url}assets/i18n.json`)).json()).en;
  const view = await (await request.get(`${gy.url}api/overview`)).json();

  await page.goto(`${gy.url}?lang=en`);
  await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(words.overviewTitle);

  const stats = main(page).getByTestId('stat');
  await expect(stats).toHaveCount(5);
  const value = key => main(page).locator(`[data-testid="stat"][data-key="${key}"]`).getByTestId('statValue');
  await expect(value('open')).toHaveText(String(view.counts.open_needs));
  await expect(value('blocked')).toHaveText(String(view.counts.blocked_needs));
  await expect(value('questions')).toHaveText(String(view.counts.open_questions));
  await expect(value('unmet')).toHaveText(String(view.counts.unmet_criteria));
  await expect(value('done')).toHaveText(String(view.counts.closed_needs));

  await expect(main(page).getByTestId('needCard')).toHaveCount(view.needs.length);
  await expect(main(page).locator('[data-testid="needCard"][data-blocked="true"]')).toHaveCount(view.needs.filter((need: { blocked: boolean }) => need.blocked).length);
  await expect(main(page).getByTestId('chain')).toHaveCount(view.blockers.length);
  await expect(main(page).getByTestId('week')).toHaveCount(view.weeks.length);
  await expect(main(page).getByTestId('overdue')).toHaveCount(view.overdue.length);
  await expect(main(page).getByTestId('unheld')).toHaveCount(Math.min(8, view.unheld.length));
});

test('a card opens its node on the detail screen', async ({ page, request }) => {
  const view = await (await request.get(`${gy.url}api/overview`)).json();
  const need = view.needs[0];
  await page.goto(`${gy.url}?lang=en`);
  await main(page).getByTestId('needCard').first().click();
  await expect(page).toHaveURL(new RegExp(`#/n/${need.id}$`));
  const node = await (await request.get(`${gy.url}api/node/${need.id}`)).json();
  await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(node.title);
});

/* A smallest whole answer: the failure scenes need a body to show after a
   retry, and the server's own answer is what the first test already matches. */
const answer = {
  seq: 1,
  at: '2026-10-30T00:00:00Z',
  counts: { open_needs: 1, blocked_needs: 0, open_questions: 0, unmet_criteria: 0, closed_needs: 0 },
  weeks: [],
  scopes: [],
  needs: [
    { id: 'n-0001', scope: 'a', title: 'a need', stage: 'before', blocked: false, last_moved: '2026-10-30T00:00:00Z', requirements: { filed: 0, approved: 0, done: 0, cancelled: 0 }, criteria: { satisfied: 0, unsatisfied: 0, withdrawn: 0 }, waiting: [] },
  ],
  blockers: [],
  deciders: [],
  overdue: [],
  unheld: [],
};

test('a refused answer keeps the frame and names the reason', async ({ page }) => {
  const words = (await (await page.request.get(`${gy.url}assets/i18n.json`)).json()).en;
  await page.route('**/api/overview', route =>
    route.fulfill({ status: 503, contentType: 'application/json', body: JSON.stringify({ reason: 'the overview took too long' }) }),
  );
  await page.goto(`${gy.url}?lang=en`);

  // The frame is there before the answer: the heading and the empty board.
  await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(words.overviewTitle);
  await expect(main(page).getByTestId('board')).toBeVisible();

  // The missing part says why, and the rest of the frame stays.
  const failed = main(page).getByTestId('failed');
  await expect(failed).toBeVisible();
  await expect(failed).toContainText('the overview took too long');
  await expect(failed).toContainText(words.loadFailed);
  await expect(main(page).getByTestId('stat')).toHaveCount(5);

  // Retry asks again; with the line back, the answer arrives.
  await page.unroute('**/api/overview');
  await page.route('**/api/overview', route =>
    route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(answer) }),
  );
  await failed.getByRole('button', { name: words.retry }).click();
  await expect(main(page).getByTestId('failed')).toHaveCount(0);
  await expect(main(page).getByTestId('needCard')).toHaveCount(1);
});

test('the frame is on screen while the overview is still out', async ({ page }) => {
  const words = (await (await page.request.get(`${gy.url}assets/i18n.json`)).json()).en;
  let release!: () => void;
  const gate = new Promise<void>(resolve => { release = resolve; });
  await page.route('**/api/overview', async route => {
    await gate;
    await route.fulfill({ status: 200, contentType: 'application/json', body: JSON.stringify(answer) });
  });

  await page.goto(`${gy.url}?lang=en`);
  // The heading and the empty board are drawn before the answer is there.
  await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(words.overviewTitle);
  await expect(main(page).getByTestId('board')).toBeVisible();
  await expect(main(page).getByTestId('loading')).toBeVisible();
  release();
  await expect(main(page).getByTestId('needCard')).toHaveCount(1);
});
