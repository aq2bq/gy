import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

const at = (page: import('@playwright/test').Page) =>
  page.evaluate(() => (window as unknown as { GyShell: { at: number | null } }).GyShell.at);

test('ArrowLeft rewinds the head and the pages answer for it', async ({ page, request }) => {
  const shell = await (await request.get(`${gy.url}api/shell`)).json();
  await page.goto(gy.url);
  await expect(page.locator('#scrub-seq')).toHaveText(`seq ${shell.seq} / ${shell.seq}`);
  expect(await at(page)).toBeNull();

  await page.keyboard.press('ArrowLeft');
  const point = shell.seq - 1;
  expect(await at(page)).toBe(point);
  await expect(page.locator('#scrub-seq')).toHaveText(`seq ${point} / ${shell.seq}`);
  // The clock leaves the canonical line and shows the point.
  await expect(page.locator('#clock')).toContainText(/viewing|見ている/);

  // A page opened by hash keeps the point and answers for it.
  await page.evaluate(() => {
    location.hash = '#/list/Need';
  });
  const rows = (await (await request.get(`${gy.url}api/list?kind=Need&at=${point}`)).json()).rows;
  const open = rows.filter((row: { status?: string }) => row.status === 'open');
  await expect(page.locator('.row')).toHaveCount(open.length);
});

test('dragging the head moves the point and now goes back', async ({ page, request }) => {
  const shell = await (await request.get(`${gy.url}api/shell`)).json();
  await page.goto(gy.url);
  await expect(page.locator('#scrub-seq')).toHaveText(/seq \d+ \/ \d+/);

  const box = await page.locator('#track').boundingBox();
  if (!box) throw new Error('the band has no box');
  const middle = box.y + box.height / 2;
  await page.mouse.move(box.x + box.width - 2, middle);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width * 0.3, middle, { steps: 5 });
  await page.mouse.up();

  const point = await at(page);
  expect(point).not.toBeNull();
  expect(point ?? 0).toBeGreaterThan(0);
  expect(point ?? 0).toBeLessThan(shell.seq);
  await expect(page.locator('#scrub-seq')).toHaveText(`seq ${point} / ${shell.seq}`);

  await page.locator('#scrub-now').click();
  await expect.poll(() => at(page)).toBeNull();
  await expect(page.locator('#scrub-seq')).toHaveText(`seq ${shell.seq} / ${shell.seq}`);
  await expect(page.locator('#clock')).toContainText(/canonical|正本/);
});
