import { expect, test, type Page } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

/* The time band (n-32a9) still names the point of the log every screen reads;
   an arrow rewinds the head and the answer is asked for at that point. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

const band = (page: Page) => page.getByTestId('band');
const at = (page: Page) => band(page).getAttribute('data-at');

test('ArrowLeft rewinds the head, and the screens answer for that point', async ({ page, request }) => {
  const shell = await (await request.get(`${gy.url}api/shell`)).json();
  await page.goto(`${gy.url}?lang=en`);
  await expect(band(page).getByText(`seq ${shell.seq} / ${shell.seq}`)).toBeVisible();
  expect(await at(page)).toBe('now');

  await page.keyboard.press('ArrowLeft');
  const point = shell.seq - 1;
  expect(await at(page)).toBe(String(point));
  await expect(band(page).getByText(`seq ${point} / ${shell.seq}`)).toBeVisible();
  await expect(page.getByTestId('seq')).toHaveText(`seq ${point} / ${shell.seq}`);

  // The first screen asks for the point, not the head.
  const asked = page.waitForRequest(request => request.url().includes(`/api/overview?at=${point}`));
  await page.goto(`${gy.url}?lang=en`);
  await page.keyboard.press('ArrowLeft');
  await asked;
});

test('the now button returns to the head', async ({ page }) => {
  await page.goto(`${gy.url}?lang=en`);
  await page.keyboard.press('ArrowLeft');
  expect(await at(page)).not.toBe('now');
  await band(page).getByRole('button', { name: /now|いま/i }).click();
  expect(await at(page)).toBe('now');
});
