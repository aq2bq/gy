import { expect, test, type Page } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

/* A write reaches the open page with no reload (n-39f2): the home's pulse, the
   sidebar's counts, the overview's needs, and the band's ticks follow the log. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

/// The newest pulse row's words, the criterion count, the band's ticks, and the
/// point the band says it is showing. The pulse is the page's own list (the
/// question cards hold plain list items too, so it is named).
const pulseList = (page: Page) => page.getByTestId('main').getByRole('list', { name: /Pulse/ });
const pulse = (page: Page) => pulseList(page).getByRole('listitem').first().innerText();
const criteria = (page: Page) => page.getByTestId('nav').getByRole('link', { name: /Criteria/ }).innerText();
const ticks = async (page: Page) => Number(await page.getByTestId('band').getAttribute('data-ticks'));
const at = (page: Page) => page.getByTestId('band').getAttribute('data-at');

test('a write updates the pulse, the sidebar, and the band', async ({ page }) => {
  await page.goto(`${gy.url}#/now`);
  await expect(page.getByTestId('band').getByText(/seq \d+ \/ \d+/)).toBeVisible();
  await expect(pulseList(page).getByRole('listitem')).toHaveCount(14);
  const before = { pulse: await pulse(page), criteria: await criteria(page), ticks: await ticks(page) };

  const started = Date.now();
  gy.gy(['--scope', 'a', 'criterion', 'add', 'the live criterion']);
  await expect(pulseList(page).getByRole('listitem').first()).not.toHaveText(before.pulse);
  await expect(page.getByTestId('nav').getByRole('link', { name: /Criteria/ })).not.toHaveText(before.criteria);
  await expect(page.getByTestId('band')).toHaveAttribute('data-ticks', String(before.ticks + 1));
  expect(Date.now() - started).toBeLessThan(1500);
});

test('a write updates the board and the band', async ({ page, request }) => {
  const nodes = (await (await request.get(`${gy.url}api/cross?scope=a`)).json()).nodes;
  const criterion = nodes.find((row: { kind: string }) => row.kind === 'Criterion');
  const view = await (await request.get(`${gy.url}api/overview`)).json();
  await page.goto(`${gy.url}?lang=en`);
  const main = page.getByTestId('main');
  const band = page.getByTestId('band');
  await expect(main.getByTestId('needCard')).toHaveCount(view.needs.length);
  const before = await ticks(page);

  const started = Date.now();
  gy.gy(['--scope', 'a', 'need', 'add', 'the live need', '--targets', criterion.id]);
  await expect(main.getByTestId('needCard')).toHaveCount(view.needs.length + 1);
  await expect(band).toHaveAttribute('data-ticks', String(before + 1));
  expect(Date.now() - started).toBeLessThan(2000);
});

test('a write leaves a rewound head where it is', async ({ page }) => {
  await page.goto(gy.url);
  const band = page.getByTestId('band');
  await expect(band.getByText(/seq \d+ \/ \d+/)).toBeVisible();
  await page.keyboard.press('ArrowLeft');
  const before = { at: await at(page), ticks: await ticks(page) };
  expect(before.at).not.toBe('now');

  gy.gy(['--scope', 'a', 'criterion', 'add', 'the second live criterion']);
  await expect(band).toHaveAttribute('data-ticks', String(before.ticks + 1));
  expect(await at(page)).toBe(before.at);
});
