import { expect, test, type Page } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

/// The band, and the point it says it is showing: the head, or "now" (n-5e43).
const band = (page: Page) => page.getByTestId('band');
const at = (page: Page) => band(page).getAttribute('data-at');

test('ArrowLeft rewinds the head and the pages answer for it', async ({ page, request }) => {
  const shell = await (await request.get(`${gy.url}api/shell`)).json();
  await page.goto(gy.url);
  await expect(band(page).getByText(`seq ${shell.seq} / ${shell.seq}`)).toBeVisible();
  expect(await at(page)).toBe('now');

  await page.keyboard.press('ArrowLeft');
  const point = shell.seq - 1;
  expect(await at(page)).toBe(String(point));
  await expect(band(page).getByText(`seq ${point} / ${shell.seq}`)).toBeVisible();
  // The clock leaves the canonical line and shows the point.
  await expect(page.getByTestId('sidebar').getByText(/viewing|見ている/)).toBeVisible();

  // A page opened by hash keeps the point and answers for it.
  await page.evaluate(() => {
    location.hash = '#/list/Need';
  });
  const rows = (await (await request.get(`${gy.url}api/list?kind=Need&at=${point}`)).json()).rows;
  const open = rows.filter((row: { status?: string }) => row.status === 'open');
  await expect(page.getByTestId('main').getByRole('link')).toHaveCount(open.length);
});

test('a stored instant reads as the same day in the reader\'s own place', async ({ browser, request }) => {
  // The newest write's node: its created day and the history heading are the
  // same write, so no midnight can fall between them.
  const history = await (await request.get(`${gy.url}api/history`)).json();
  const node = history.rows[0].node as string;
  const day = /\d{4}\/\d{2}\/\d{2}/;

  const inZone = async (timezoneId: string) => {
    const context = await browser.newContext({ timezoneId });
    const page = await context.newPage();
    await page.goto(`${gy.url}#/n/${node}`);
    const created = (await page.getByTestId('created').innerText()).match(day)![0];
    await page.goto(`${gy.url}#/history`);
    const heading = await page.getByTestId('hist-day').first().innerText();
    await context.close();
    return { created, heading };
  };

  // 26 hours apart, so the calendar day always differs.
  const east = await inZone('Pacific/Kiritimati'); // UTC+14
  const west = await inZone('Etc/GMT+12'); // UTC-12
  expect(east.created).toBe(east.heading);
  expect(west.created).toBe(west.heading);
  expect(east.created).not.toBe(west.created);
});

test('dragging the head moves the point and now goes back', async ({ page, request }) => {
  const shell = await (await request.get(`${gy.url}api/shell`)).json();
  await page.goto(gy.url);
  await expect(band(page).getByText(/seq \d+ \/ \d+/)).toBeVisible();

  const track = band(page).getByRole('slider');
  const box = await track.boundingBox();
  if (!box) throw new Error('the band has no box');
  const middle = box.y + box.height / 2;
  await page.mouse.move(box.x + box.width - 2, middle);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.3, middle, { steps: 5 });
  await page.mouse.up();

  const point = await at(page);
  expect(point).not.toBe('now');
  expect(Number(point)).toBeGreaterThan(0);
  expect(Number(point)).toBeLessThan(shell.seq);
  await expect(band(page).getByText(`seq ${point} / ${shell.seq}`)).toBeVisible();

  await band(page).getByRole('button', { name: 'now' }).click();
  await expect.poll(() => at(page)).toBe('now');
  await expect(band(page).getByText(`seq ${shell.seq} / ${shell.seq}`)).toBeVisible();
  await expect(page.getByTestId('sidebar').getByText(/canonical|正本/)).toBeVisible();
});