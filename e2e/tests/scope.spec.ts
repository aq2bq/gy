import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('a scope wears one badge everywhere, and only a scope wears it', async ({ page, request }) => {
  const shell = await (await request.get(`${gy.url}api/shell`)).json();
  const names: string[] = shell.scopes.map((item: { name: string }) => item.name);
  const rows = (await (await request.get(`${gy.url}api/list?kind=Need`)).json()).rows;
  // The list opens on the open rows, newest first: the first row shown is the
  // first open row in the answer, which is not always the answer's first row.
  const open = rows.filter((row: { status?: string }) => row.status === 'open');
  const shown = open[0].scope as string;

  await page.goto(`${gy.url}#/list/Need`);
  const scopes = page.getByTestId('scopes');

  // In one ledger no two scopes share a hue, so the frames tell them apart.
  const sidebarColors = await Promise.all(
    names.map(name => scopes.getByText(name, { exact: true }).evaluate(el => getComputedStyle(el).borderColor)),
  );
  expect(sidebarColors.length).toBeGreaterThan(1);
  expect(new Set(sidebarColors).size).toBe(sidebarColors.length);

  const row = page.getByTestId('main').getByRole('link').first();
  const badge = row.getByText(shown, { exact: true });
  await expect(badge).toHaveCount(1);
  const listColor = await badge.evaluate(el => getComputedStyle(el).borderColor);

  // The created date keeps its plain cell: one badge per row, for the scope.
  await expect(row.getByText(shown, { exact: true })).toHaveCount(1);

  // The same node's page wears the same badge in the same colour.
  await row.click();
  await expect(page).toHaveURL(/#\/n\//);
  const meta = page.getByTestId('main').getByText(shown, { exact: true });
  await expect(meta).toHaveText(shown);
  expect(await meta.evaluate(el => getComputedStyle(el).borderColor)).toBe(listColor);

  // "all scopes" is not a scope's name, so it stays plain.
  await expect(scopes.getByRole('button', { name: /all/ }).getByText(shown, { exact: true })).toHaveCount(0);
});