import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

/* A write reaches the open page with no reload (n-39f2): the overview's needs
   and the band's ticks follow the log. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('a write updates the board and the band', async ({ page, request }) => {
  const nodes = (await (await request.get(`${gy.url}api/cross?scope=a`)).json()).nodes;
  const criterion = nodes.find((row: { kind: string }) => row.kind === 'Criterion');
  const view = await (await request.get(`${gy.url}api/overview`)).json();
  await page.goto(`${gy.url}?lang=en`);
  const main = page.getByTestId('main');
  const band = page.getByTestId('band');
  await expect(main.getByTestId('needCard')).toHaveCount(view.needs.length);
  const ticks = Number(await band.getAttribute('data-ticks'));

  const started = Date.now();
  gy.gy(['--scope', 'a', 'need', 'add', 'the live need', '--targets', criterion.id]);
  await expect(main.getByTestId('needCard')).toHaveCount(view.needs.length + 1);
  await expect(band).toHaveAttribute('data-ticks', String(ticks + 1));
  expect(Date.now() - started).toBeLessThan(2000);
});

test('a write leaves a rewound head where it is', async ({ page }) => {
  await page.goto(`${gy.url}?lang=en`);
  const band = page.getByTestId('band');
  await expect(band.getByText(/seq \d+ \/ \d+/)).toBeVisible();
  await page.keyboard.press('ArrowLeft');
  const before = { at: await band.getAttribute('data-at'), ticks: Number(await band.getAttribute('data-ticks')) };
  expect(before.at).not.toBe('now');

  gy.gy(['--scope', 'a', 'criterion', 'add', 'the second live criterion']);
  await expect(band).toHaveAttribute('data-ticks', String(before.ticks + 1));
  expect(await band.getAttribute('data-at')).toBe(before.at);
});
