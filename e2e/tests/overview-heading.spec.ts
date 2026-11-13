/* r-5a94 / ac-08c8: the heading distinguishes the selected scope from the
 * ledger directory and follows the URL even while reads are pending. */
import { expect, test, type Page } from '@playwright/test';
import { basename } from 'node:path';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;
const views: Record<string, any> = {};
test.beforeAll(async () => {
  gy = await start();
  expect(['a', 'b']).not.toContain(basename(gy.dir));
  for (const scope of ['a', 'b']) views[scope] = await (await fetch(`${gy.url}api/overview?scope=${scope}`)).json();
});
test.afterAll(async () => { await gy?.stop(); });

const main = (page: Page) => page.getByTestId('main');
const scopeLabel = (page: Page) => main(page).getByTestId('overviewScope');
const pick = (page: Page, scope: string) => page.getByTestId('scopes').locator(`button[data-s="${scope}"]`).click();
const value = (page: Page, key: string) => main(page).locator(`[data-testid="stat"][data-key="${key}"]`).getByTestId('statValue');
const fill = (text: string, values: Record<string, string>) => text.replace(/\{(\w+)\}/g, (_, key) => values[key]);

for (const [lang, label, checking, empty, heading] of [
  ['ja', 'スコープ: ', 'スコープ（確認中）: ', 'ノードがないスコープ: ', '進行の全体像'],
  ['en', 'Scope: ', 'Scope (checking): ', 'Scope with no nodes: ', 'Where the work stands'],
]) {
  async function matches(page: Page, scope: string) {
    await expect(scopeLabel(page)).toHaveText(`${label}${scope}`);
    await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(heading);
    await expect(main(page).getByTestId('overviewSub')).toContainText(basename(gy.dir));
    await expect(main(page).getByTestId('overviewSub')).toContainText(`seq ${views[scope].seq}`);
    for (const [key, field] of Object.entries({ open: 'open_needs', blocked: 'blocked_needs', questions: 'open_questions', unmet: 'unmet_criteria', done: 'closed_needs' })) {
      await expect(value(page, key)).toHaveText(String(views[scope].counts[field]));
    }
    await expect(main(page).getByTestId('needCard')).toHaveCount(views[scope].needs.length);
    await expect(page).toHaveTitle(`gy - ${basename(gy.dir)}`);
    await expect(main(page).getByTestId('overviewSub').getByTestId('scopeBadge')).toHaveCount(0);
  }

  test(`${lang}: scope-labelled heading follows choices and back/forward on both progress URLs`, async ({ page }) => {
    for (const route of ['#/', '#/overview']) {
      await page.goto(`${gy.url}?lang=${lang}${route}?only=a`);
      await matches(page, 'a');
      await page.evaluate(() => { window['headingMarker'] = 'same document'; });
      await pick(page, 'b');
      await matches(page, 'b');
      await page.goBack();
      await expect(page).toHaveURL(/only=a$/);
      await matches(page, 'a');
      await page.goForward();
      await expect(page).toHaveURL(/only=b$/);
      await matches(page, 'b');
      expect(await page.evaluate(() => window['headingMarker'])).toBe('same document');
    }
  });

  test(`${lang}: no selection keeps the original heading, ledger, sequence and instant`, async ({ page, request }) => {
    const words = (await (await request.get(`${gy.url}assets/i18n.json`)).json())[lang];
    const view = await (await request.get(`${gy.url}api/overview`)).json();
    for (const route of ['#/', '#/overview']) {
      await page.goto(`${gy.url}?lang=${lang}${route}`);
      await expect(value(page, 'open')).toHaveText(String(view.counts.open_needs));
      await expect(scopeLabel(page)).toHaveCount(0);
      await expect(main(page).getByRole('heading', { level: 1 })).toHaveText(heading);
      const instant = await page.evaluate(({ at, lang }) => new Date(at).toLocaleString(lang === 'ja' ? 'ja-JP' : 'en-GB', {
        month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit',
      }), { at: view.at, lang });
      await expect(main(page).getByTestId('overviewSub')).toHaveText(fill(words.overviewSub, {
        ledger: basename(gy.dir), seq: String(view.seq), at: instant,
      }));
    }
  });

  test(`${lang}: the URL scope is explicit before reads, and old counts disappear on a switch`, async ({ page }) => {
    let shell!: () => void, overview!: () => void, next!: () => void;
    const shellGate = new Promise<void>(resolve => { shell = resolve; });
    const overviewGate = new Promise<void>(resolve => { overview = resolve; });
    const nextGate = new Promise<void>(resolve => { next = resolve; });
    await page.route('**/api/shell?scope=a', async route => { await shellGate; await route.continue(); });
    await page.route('**/api/overview?scope=a', async route => { await overviewGate; await route.continue(); });
    await page.route('**/api/overview?scope=b', async route => { await nextGate; await route.continue(); });
    try {
      await page.goto(`${gy.url}?lang=${lang}#/?only=a`);
      await expect(scopeLabel(page)).toHaveText(`${checking}a`);
      await expect(value(page, 'open')).toHaveText('–');
      shell();
      await expect(scopeLabel(page)).toHaveText(`${label}a`);
      await expect(value(page, 'open')).toHaveText('–');
      overview();
      await matches(page, 'a');
      expect(views.a.counts.open_needs).not.toBe(views.b.counts.open_needs);
      await pick(page, 'b');
      await expect(scopeLabel(page)).toHaveText(`${label}b`);
      await expect(value(page, 'open')).toHaveText('–');
      await expect(main(page).getByTestId('needCard')).toHaveCount(0);
      next();
      await matches(page, 'b');
    } finally { shell(); overview(); next(); }
  });

  test(`${lang}: a name with no nodes is not shown as a confirmed scope`, async ({ page }) => {
    const name = 'missing <scope>';
    let shell!: () => void;
    const shellGate = new Promise<void>(resolve => { shell = resolve; });
    await page.route('**/api/shell?scope=*', async route => { await shellGate; await route.continue(); });
    try {
      await page.goto(`${gy.url}?lang=${lang}#/?only=${encodeURIComponent(name)}`);
      await expect(scopeLabel(page)).toHaveText(`${checking}${name}`);
      await expect(value(page, 'open')).toHaveText('0');
      shell();
      await expect(scopeLabel(page)).toHaveText(`${empty}${name}`);
      await expect(scopeLabel(page).locator('scope')).toHaveCount(0);
      await expect(page.getByTestId('unknownScope')).toContainText(name);
      await page.getByTestId('unknownScope').getByRole('button').click();
      await expect(scopeLabel(page)).toHaveCount(0);
    } finally { shell(); }
  });
}
