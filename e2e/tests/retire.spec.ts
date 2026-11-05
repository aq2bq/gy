import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

const mark = 'UNIQUE_RETIRE_PAYLOAD';
let ledger: Ledger;
let hidden: string[] = [];
let visible: string;
let oldSeq: number;

test.beforeAll(async () => {
  ledger = await start({ build: gy => {
    const id = (args: string[]) => JSON.parse(gy(['--scope', 'b', ...args])).id as string;
    const ac = id(['criterion', 'add', mark]);
    const need = id(['need', 'add', mark, '--targets', ac]);
    const req = id(['req', 'add', mark, '--need', need, '--targets', ac]);
    gy(['req', 'approve', req, '--design', mark, '--heard-by', 'master', '--evidence', mark]);
    gy(['criterion', 'satisfy', ac, '--evidence', mark]);
    gy(['req', 'done', req, '--evidence', mark]);
    const q = id(['question', 'add', mark, '--decider', 'master', '--options', 'a', '--options', 'b']);
    gy(['question', 'close', q, '--by', 'fact', '--evidence', mark]);
    const d = id(['decide', mark, '--scope-note', mark, '--serves', need]);
    hidden = [ac, need, req, q, d];
    const live = JSON.parse(gy(['--scope', 'a', 'criterion', 'add', 'visible criterion'])).id;
    visible = JSON.parse(gy(['--scope', 'a', 'need', 'add', 'visible need', '--targets', live])).id;
  } });
});
test.afterAll(async () => { await ledger?.stop(); });

test('all GET readings and browser screens exclude retired payloads, including past points', async ({ page, request }) => {
  oldSeq = (await (await request.get(`${ledger.url}api/shell`)).json()).seq;
  const routes = ['shell', 'now', 'overview', 'cross', 'list', 'history', 'labels', 'search?q=' + mark,
    'ticks', `node/${visible}`, `node/${visible}/path`, ...hidden.flatMap(id => [`node/${id}`, `node/${id}/path`])];
  const before = new Map<string, string>();
  for (const route of routes) before.set(route, await (await request.get(`${ledger.url}api/${route}`)).text());
  expect(before.get('history')).toContain(mark);
  for (const id of hidden) expect(before.get(`node/${id}`)).toContain(mark);
  await page.goto(`${ledger.url}#/history`);
  await expect(page.getByTestId('main')).toContainText(mark);
  ledger.gy(['retire', ...hidden, '--reason', 'earlier process records']);
  for (const route of routes) {
    for (const at of ['', String(oldSeq)]) {
      const query = at ? `${route.includes('?') ? '&' : '?'}at=${at}` : '';
      const body = await (await request.get(`${ledger.url}api/${route}${query}`)).text();
      expect(body, `${route} at ${at}`).not.toContain(mark);
      // Only the retirement write in history may name the retired IDs.
      if (route === 'history' && !at) {
        const rows = JSON.parse(body).rows;
        const retired = rows.filter((row: { what: string }) => row.what.startsWith('retired '));
        expect(retired).toHaveLength(1);
        expect(retired[0].source).toBe('earlier process records');
        for (const id of hidden) expect(retired[0].what).toContain(id);
        for (const row of rows.filter((row: { what: string }) => !row.what.startsWith('retired '))) {
          for (const id of hidden) expect(JSON.stringify(row)).not.toContain(id);
        }
      }
      if (route !== 'history') for (const id of hidden) expect(body, `${route} at ${at}`).not.toContain(id);
    }
  }
  for (const hash of ['/', '/now', '/history', '/cross', '/list/Need', '/list/Decision', `/n/${visible}`, ...hidden.map(id => `/n/${id}`)]) {
    await page.goto(`${ledger.url}#${hash}`);
    await expect(page.getByTestId('main')).toBeVisible();
    await expect(page.locator('body')).not.toContainText(mark);
  }
  // Time travel on a visible node still uses the current retirement boundary.
  await page.goto(`${ledger.url}#/n/${visible}`);
  await expect(page.getByTestId('band').getByText(/seq \d+ \/ \d+/)).toBeVisible();
  await page.keyboard.press('ArrowLeft');
  await expect(page.locator('body')).not.toContainText(mark);
});
