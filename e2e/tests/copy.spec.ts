import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the ID beside a node copies, and its sign comes and goes', async ({ page, context, request }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);
  const nodes = (await (await request.get(`${gy.url}api/cross?q=closes`)).json()).nodes;
  const decision = nodes.find((row: { kind: string; title: string }) => row.kind === 'Decision' && row.title.startsWith('closes'));

  await page.goto(`${gy.url}#/n/${decision.id}`);
  const mark = page.getByTestId('main').getByRole('button', { name: new RegExp(`Copy ${decision.id}`) });
  await expect(mark).toBeVisible();

  await mark.click();
  expect(await page.evaluate(() => navigator.clipboard.readText())).toBe(decision.id);
  await expect(mark).toHaveText('✓');
  // The sign goes away on its own: the root's timer clears it (n-6afd).
  await expect(mark).not.toHaveText('✓', { timeout: 3000 });
});
