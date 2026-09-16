import { expect, test, type Page } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

/// The newest pulse row's words, the criterion count, the band's tick marks,
/// and the point the band says it is showing. The pulse is the page's own list
/// (the question cards hold plain list items too, so it is named).
const pulseList = (page: Page) => page.getByTestId('main').getByRole('list', { name: /Pulse/ });
const pulse = (page: Page) => pulseList(page).getByRole('listitem').first().innerText();
const criteria = (page: Page) => page.getByTestId('nav').getByRole('link', { name: /Criteria/ }).innerText();
const ticks = (page: Page) => page.getByTestId('band').locator('rect').count();
const at = (page: Page) => page.getByTestId('band').getAttribute('data-at');

test('a write updates the pulse, the sidebar, and the band', async ({ page }) => {
  await page.goto(gy.url);
  await expect(page.getByTestId('band').getByText(/seq \d+ \/ \d+/)).toBeVisible();
  await expect(pulseList(page).getByRole('listitem')).toHaveCount(14);
  const before = { pulse: await pulse(page), criteria: await criteria(page), ticks: await ticks(page) };

  const started = Date.now();
  gy.gy(['--scope', 'a', 'criterion', 'add', 'the live criterion']);
  await expect(pulseList(page).getByRole('listitem').first()).not.toHaveText(before.pulse);
  await expect(page.getByTestId('nav').getByRole('link', { name: /Criteria/ })).not.toHaveText(before.criteria);
  await expect(page.getByTestId('band').locator('rect')).toHaveCount(before.ticks + 1);
  expect(Date.now() - started).toBeLessThan(1500);
});

test('a write leaves a rewound head where it is', async ({ page }) => {
  await page.goto(gy.url);
  // The band must know its end before a key can rewind from it.
  await expect(page.getByTestId('band').getByText(/seq \d+ \/ \d+/)).toBeVisible();
  await page.keyboard.press('ArrowLeft');
  const before = { at: await at(page), ticks: await ticks(page) };
  expect(before.at).not.toBe('now');

  gy.gy(['--scope', 'a', 'criterion', 'add', 'the second live criterion']);
  // The band takes the new write; the point stays.
  await expect(page.getByTestId('band').locator('rect')).toHaveCount(before.ticks + 1);
  expect(await at(page)).toBe(before.at);
});