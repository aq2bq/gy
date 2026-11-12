/* The sidebar's scope in the URL (r-d060, ac-1ecf, ac-2ffe, ac-bbcc): the
   place names the scope as the query `only`, so a fresh tab, a back step, and
   every link keep it. "All scopes" is the absence of the name, and a crossing
   keeps its own `scope` list apart from it (d-2b18). */
import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

const main = (page: Page) => page.getByTestId('main');
const scopes = (page: Page) => page.getByTestId('scopes');
const scopeButton = (page: Page, name: string) => scopes(page).locator(`button[data-s="${name}"]`);
const hash = (page: Page) => new URL(page.url()).hash;

async function pick(page: Page, name: string) {
  await scopeButton(page, name).click();
}

/** Whether the sidebar draws that scope as chosen: the button's own mark. */
async function selected(page: Page, name: string) {
  return scopeButton(page, name).evaluate(el => el.classList.contains('on'));
}

/** A kind's count as the sidebar draws it: decisions count all, the rest open/all. */
function countLine(row: { kind: string; open: number; total: number }) {
  return row.kind === 'Decision' ? `${row.total}` : `${row.open} / ${row.total}`;
}

/** The sidebar's per-kind counts must be the ones `/api/shell` answers for the
 *  scope the URL names: the counts are the scope's, like the rows and the mark. */
async function kindsMatch(page: Page, request: APIRequestContext, scope: string) {
  const shell = await (await request.get(`${gy.url}api/shell?scope=${scope}`)).json();
  expect(shell.scope).toBe(scope);
  for (const row of shell.kinds as Array<{ kind: string; open: number; total: number }>) {
    const cell = page.getByTestId('nav').locator(`[data-testid="kindCount"][data-kind="${row.kind}"]`);
    await expect(cell).toHaveText(countLine(row));
  }
}

test('a URL without a scope opens the same screen, unchanged', async ({ page, request }) => {
  const need = (await (await request.get(`${gy.url}api/list?kind=Need`)).json()).rows[0];
  const words = (await (await request.get(`${gy.url}assets/i18n.json`)).json()).en;
  const cases: Array<[string, () => Promise<void>]> = [
    ['#/', async () => { await expect(main(page).getByTestId('board')).toBeVisible(); }],
    ['#/overview', async () => { await expect(main(page).getByTestId('board')).toBeVisible(); }],
    ['#/now', async () => { await expect(main(page).locator('[data-eye="wait"]')).toBeVisible(); }],
    ['#/cross', async () => { await expect(main(page).getByTestId('crossWord')).toBeVisible(); }],
    ['#/list/Need', async () => { await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(words.Needs); }],
    [`#/n/${need.id}`, async () => { await expect(main(page).locator('[data-shown]')).toBeVisible(); }],
    ['#/eye/wait', async () => { await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(words.eyeWait); }],
    ['#/history', async () => { await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(words.histTitle); }],
  ];
  for (const [where, ready] of cases) {
    await page.goto(`${gy.url}?lang=en${where}`);
    await ready();
    // The shell has answered, so a rewrite would already have happened.
    await expect(scopeButton(page, 'all')).toHaveClass(/on/);
    expect(hash(page)).toBe(where);
    await expect(page.getByTestId('unknownScope')).toHaveCount(0);
  }
});

test('a crossing that names scopes keeps its URL, and the sidebar stays on all', async ({ page, request }) => {
  const cut = await (await request.get(`${gy.url}api/cross?scope=a&scope=b`)).json();
  await page.goto(`${gy.url}?lang=en#/cross?scope=a&scope=b`);
  await expect(main(page).getByTestId('laneNode')).toHaveCount(cut.nodes.length);
  expect(hash(page)).toBe('#/cross?scope=a&scope=b');
  // The crossing's own chips are the ones chosen; the sidebar's choice is not.
  await expect(main(page).getByTestId('scopeChip').filter({ hasText: /^a/ })).toHaveClass(/on/);
  await expect(main(page).getByTestId('scopeChip').filter({ hasText: /^b/ })).toHaveClass(/on/);
  await expect(scopeButton(page, 'all')).toHaveClass(/on/);
});

test('a crossing narrowing and the sidebar scope do not overwrite each other', async ({ page, request }) => {
  const cut = await (await request.get(`${gy.url}api/cross?scope=a`)).json();
  await page.goto(`${gy.url}?lang=en#/cross?scope=a`);
  await expect(main(page).getByTestId('laneNode')).toHaveCount(cut.nodes.length);

  await pick(page, 'b');
  await expect(page).toHaveURL(/#\/cross\?scope=a&only=b$/);
  // The picture is still the crossing's own cut; the sidebar is the other b.
  await expect(main(page).getByTestId('laneNode')).toHaveCount(cut.nodes.length);
  await expect(main(page).getByTestId('scopeChip').filter({ hasText: /^a/ })).toHaveClass(/on/);
  await expect(main(page).getByTestId('scopeChip').filter({ hasText: /^b/ })).not.toHaveClass(/on/);
  expect(await selected(page, 'b')).toBe(true);
});

test('the chosen scope is in the URL, and a fresh tab reads the same rows', async ({ page, browser, request }) => {
  const decisions = await (await request.get(`${gy.url}api/list?kind=Decision&scope=b`)).json();
  expect(decisions.rows.length).toBeGreaterThan(0);

  await page.goto(`${gy.url}?lang=en#/list/Decision`);
  await pick(page, 'b');
  await expect(page).toHaveURL(/#\/list\/Decision\?only=b$/);
  await expect(main(page).getByRole('link')).toHaveCount(decisions.rows.length);
  const badges = await main(page).getByTestId('scopeBadge').all();
  expect(badges).toHaveLength(decisions.rows.length);
  for (const badge of badges) await expect(badge).toHaveText('b');

  // The place is the whole choice: a tab that only knows the URL reads the
  // same rows, with the same scope chosen.
  const other = await browser.newContext();
  const tab = await other.newPage();
  await tab.goto(page.url());
  const there = tab.getByTestId('scopes').locator('button[data-s="b"]');
  await expect(there).toHaveClass(/on/);
  await expect(tab.getByTestId('main').getByRole('link')).toHaveCount(decisions.rows.length);
  await expect(tab.getByTestId('unknownScope')).toHaveCount(0);
  await other.close();

  // The same URL rule on the history page and on one node.
  const writes = await (await request.get(`${gy.url}api/history?scope=b`)).json();
  await page.goto(`${gy.url}?lang=en#/history?only=b`);
  await expect(scopeButton(page, 'b')).toHaveClass(/on/);
  await expect(main(page).getByRole('listitem')).toHaveCount(writes.rows.length);

  const node = decisions.rows[0].id;
  await page.goto(`${gy.url}?lang=en#/n/${node}?only=b`);
  await expect(main(page).locator('[data-shown]')).toBeVisible();
  await expect(scopeButton(page, 'b')).toHaveClass(/on/);
  await expect(main(page).getByTestId('nodeMeta').getByText('b', { exact: true })).toHaveCount(1);
});

test('back and forward move the scope with the screen', async ({ page, request }) => {
  const a = await (await request.get(`${gy.url}api/list?kind=Decision&scope=a`)).json();
  const b = await (await request.get(`${gy.url}api/list?kind=Decision&scope=b`)).json();
  await page.goto(`${gy.url}?lang=en#/list/Decision`);
  await pick(page, 'a');
  await expect(page).toHaveURL(/#\/list\/Decision\?only=a$/);
  await pick(page, 'b');
  await expect(page).toHaveURL(/#\/list\/Decision\?only=b$/);

  await page.goBack();
  await expect(page).toHaveURL(/#\/list\/Decision\?only=a$/);
  await expect(scopeButton(page, 'a')).toHaveClass(/on/);
  await expect(main(page).getByRole('link')).toHaveCount(a.rows.length);
  // The sidebar's counts are read under the scope too: back changes them.
  await kindsMatch(page, request, 'a');

  await page.goForward();
  await expect(page).toHaveURL(/#\/list\/Decision\?only=b$/);
  await expect(scopeButton(page, 'b')).toHaveClass(/on/);
  await expect(main(page).getByRole('link')).toHaveCount(b.rows.length);
  await kindsMatch(page, request, 'b');

  // A place between two choices: back lands on the node with the old scope,
  // then on the list that opened it.
  await page.goto(`${gy.url}?lang=en#/list/Decision?only=a`);
  await main(page).getByRole('link').first().click();
  await expect(page).toHaveURL(/#\/n\/[^?]+\?only=a$/);
  await expect(scopeButton(page, 'a')).toHaveClass(/on/);
  await pick(page, 'b');
  await expect(page).toHaveURL(/#\/n\/[^?]+\?only=b$/);

  await page.goBack();
  await expect(page).toHaveURL(/#\/n\/[^?]+\?only=a$/);
  await expect(main(page).locator('[data-shown]')).toBeVisible();
  await expect(scopeButton(page, 'a')).toHaveClass(/on/);
  await kindsMatch(page, request, 'a');
  await page.goBack();
  await expect(page).toHaveURL(/#\/list\/Decision\?only=a$/);
});

test('a scope the ledger does not have says so, and all scopes comes back', async ({ page, request }) => {
  const all = await (await request.get(`${gy.url}api/list?kind=Decision`)).json();
  await page.goto(`${gy.url}?lang=en#/list/Decision?only=nope`);

  const warn = page.getByTestId('unknownScope');
  await expect(warn).toBeVisible();
  await expect(warn).toContainText('nope');
  // Nothing is read under the unknown name: no rows, and nothing chosen.
  await expect(main(page).getByRole('link')).toHaveCount(0);
  for (const name of ['all', 'a', 'b']) await expect(scopeButton(page, name)).not.toHaveClass(/on/);

  await warn.getByRole('button').click();
  await expect(page).toHaveURL(/#\/list\/Decision$/);
  await expect(warn).toHaveCount(0);
  await expect(scopeButton(page, 'all')).toHaveClass(/on/);
  await expect(main(page).getByRole('link')).toHaveCount(all.rows.length);
});

test('every way out of a page keeps the chosen scope', async ({ page, request }) => {
  const words = (await (await request.get(`${gy.url}assets/i18n.json`)).json()).en;
  const decisions = await (await request.get(`${gy.url}api/list?kind=Decision&scope=b`)).json();
  const node = decisions.rows[0].id;
  const entry = (name: string) => page.getByTestId('navLabel').filter({ hasText: new RegExp(`^${name}$`) });

  await page.goto(`${gy.url}?lang=en#/list/Decision?only=b`);
  await main(page).getByRole('link').first().click();
  await expect(page).toHaveURL(new RegExp(`#/n/${node}\\?only=b$`));

  // The breadcrumb's kind list.
  await page.getByTestId('topbar').getByRole('link', { name: words.Decisions }).click();
  await expect(page).toHaveURL(/#\/list\/Decision\?only=b$/);

  // The entrances and the eyes.
  await entry(words.tabOverview).click();
  await expect(page).toHaveURL(/#\/\?only=b$/);
  await entry(words.now).click();
  await expect(page).toHaveURL(/#\/now\?only=b$/);
  await page.getByTestId('eyes').getByRole('link').first().click();
  await expect(page).toHaveURL(/#\/eye\/wait\?only=b$/);

  // The wordmark returns to the first screen with the scope.
  await page.getByTestId('ledger').click();
  await expect(page).toHaveURL(/#\/\?only=b$/);

  // The palette's hit.
  // d-e78c: URL changes before hashchange has finished drawing the destination.
  await expect(main(page).getByTestId('board')).toBeVisible();
  await page.keyboard.press('Control+k');
  const palette = page.getByTestId('palette');
  await expect(palette).toBeVisible();
  await palette.getByRole('textbox').fill(node);
  await expect(palette.getByRole('option').first()).toContainText(node);
  await page.keyboard.press('Enter');
  await expect(page).toHaveURL(new RegExp(`#/n/${node}\\?only=b$`));
});
