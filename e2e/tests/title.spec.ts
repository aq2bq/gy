import { expect, test } from '@playwright/test';
import { basename } from 'node:path';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the tab names the ledger directory', async ({ page }) => {
  await page.goto(gy.url);
  await expect(page).toHaveTitle(`gy - ${basename(gy.dir)}`);
});

test('the sidebar names the ledger on every page', async ({ page, request }) => {
  const rows = (await (await request.get(`${gy.url}api/list?kind=Need`)).json()).rows;
  const id = rows[0].id;
  const title = `gy - ${basename(gy.dir)}`;
  for (const route of ['#/', '#/list/Need', '#/graph', '#/history', `#/n/${id}`, '#/eye/wait']) {
    await page.goto(`${gy.url}${route}`);
    await expect(page).toHaveTitle(title);
    // The visible name is the one the title carries, not another derivation.
    await expect(page.getByTestId('ledger')).toHaveText(title.replace(/^gy - /, ''));
  }
});

test('the ledger name is a badge, at both widths', async ({ page }) => {
  for (const width of [1600, 900]) {
    await page.setViewportSize({ width, height: 720 });
    await page.goto(gy.url);
    const name = page.getByTestId('ledger');
    await expect(name).toBeVisible();
    await expect(name).toHaveText(basename(gy.dir));
    await expect(page).toHaveTitle(`gy - ${basename(gy.dir)}`);

    // A badge of its own: its own ground and edge, and neither a scope's pill
    // nor a kind's badge (n-c795b1).
    const style = await name.evaluate(el => {
      const said = getComputedStyle(el);
      return { border: said.borderTopWidth, background: said.backgroundColor, cls: el.className };
    });
    expect(style.border).not.toBe('0px');
    expect(style.background).not.toBe('rgba(0, 0, 0, 0)');
    expect(style.cls).not.toContain('sb');
    expect(style.cls).not.toContain('k');
  }
});

test('two ledgers name themselves apart', async ({ page }) => {
  const other = await start();
  try {
    await page.goto(gy.url);
    const first = await page.getByTestId('ledger').innerText();
    await page.goto(other.url);
    const second = await page.getByTestId('ledger').innerText();
    expect(first).toBe(basename(gy.dir));
    expect(second).toBe(basename(other.dir));
    expect(second).not.toBe(first);
  } finally {
    await other.stop();
  }
});