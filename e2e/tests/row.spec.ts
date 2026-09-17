import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;
let criterion: string;

test.beforeAll(async () => {
  // A criterion (a write with a node) and a scope rename (a write without one).
  gy = await start({
    build: (run) => {
      criterion = JSON.parse(run(['--scope', 'a', 'criterion', 'add', 'a criterion'])).id as string;
      run(['scope', 'rename', 'a', 'c']);
    },
  });
});

test.afterAll(async () => {
  await gy?.stop();
});

test('a row opens its node, and a row without one stays', async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 900 });
  await page.goto(gy.url);
  const rail = page.getByTestId('railBody');
  await expect(rail.getByRole('listitem')).toHaveCount(2);

  // The clock's cell is a hit now: the whole row is the link.
  const row = rail.getByRole('listitem').filter({ hasText: 'criterion add' }).first();
  const box = await row.boundingBox();
  if (!box) throw new Error('the row has no box');
  await page.mouse.click(box.x + 4, box.y + 6);
  await expect(page).toHaveURL(new RegExp(`#/n/${criterion}$`));

  // A write without a node (a scope rename) does not move.
  await page.goto(gy.url);
  const rename = rail.getByRole('listitem').filter({ hasText: 'scope rename' }).first();
  const before = page.url();
  const other = await rename.boundingBox();
  if (!other) throw new Error('the row has no box');
  await page.mouse.click(other.x + 4, other.y + 6);
  await page.waitForTimeout(300);
  expect(page.url()).toBe(before);
});