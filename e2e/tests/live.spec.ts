import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

const shape = (page: import('@playwright/test').Page) =>
  page.evaluate(() => ({
    pulse: (document.querySelector('.pi .when') as HTMLElement | null)?.innerText ?? '',
    criteria: (document.querySelector('#nav a[href="#/list/Criterion"] .cnt') as HTMLElement | null)
      ?.innerText ?? '',
    ticks: document.querySelectorAll('#ticks rect').length,
    at: (window as unknown as { GyShell: { at: number | null } }).GyShell.at,
  }));

test('a write updates the pulse, the sidebar, and the band', async ({ page }) => {
  await page.goto(gy.url);
  await expect(page.locator('#scrub-seq')).toHaveText(/seq \d+ \/ \d+/);
  await expect(page.locator('.pi')).toHaveCount(14);
  const before = await shape(page);

  const started = Date.now();
  gy.gy(['--scope', 'a', 'criterion', 'add', 'the live criterion']);
  await expect(page.locator('.pi .when').first()).not.toHaveText(before.pulse);
  await expect(page.locator('#nav a[href="#/list/Criterion"] .cnt')).not.toHaveText(before.criteria);
  await expect(page.locator('#ticks rect')).toHaveCount(before.ticks + 1);
  expect(Date.now() - started).toBeLessThan(1500);
});

test('a write leaves a rewound head where it is', async ({ page }) => {
  await page.goto(gy.url);
  // The band must know its end before a key can rewind from it.
  await expect(page.locator('#scrub-seq')).toHaveText(/seq \d+ \/ \d+/);
  await page.keyboard.press('ArrowLeft');
  const before = await shape(page);
  expect(before.at).not.toBeNull();

  gy.gy(['--scope', 'a', 'criterion', 'add', 'the second live criterion']);
  // The band takes the new write; the point stays.
  await expect(page.locator('#ticks rect')).toHaveCount(before.ticks + 1);
  expect(await shape(page)).toMatchObject({ at: before.at });
});
