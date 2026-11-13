/* Progress follows the sidebar scope (r-e9ab). CLI rows are the independent
 * count oracle; both scopes have direct and transitive cross-scope waits. */
import { expect, test, type APIRequestContext, type Page } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

type Row = { id: string; kind: string; status?: string; title: string; scope: string; unwaited?: boolean };
type Group = { direct: string; chain: string; middle: string; root: string };
let gy: Ledger;
const groups: Record<string, Group> = {};
let all: Row[];
const cli: Record<string, { rows: Row[]; counts: Record<string, number> }> = {};

function build(run: Ledger['gy']) {
  const add = (scope: string, args: string[]) => JSON.parse(run(['--scope', scope, ...args])).id as string;
  for (const [scope, extra] of [['a', 1], ['b', 2]] as const) {
    const target = add(scope, ['criterion', 'add', `${scope} target`]);
    run(['edit', target, '--reason', 'past due', '--set', 'due=2020-01-01']);
    const need = (name: string) => add(scope, ['need', 'add', `${scope} ${name}`, '--targets', target]);
    groups[scope] = {
      direct: need('direct'), chain: need('chain'), middle: need('middle'),
      root: add(scope, ['question', 'add', `${scope} root`, '--decider', `${scope} decider`, '--options', 'yes', '--options', 'no']),
    };
    for (let i = 0; i < extra; i++) {
      const orphan = add(scope, ['criterion', 'add', `${scope} orphan check ${i}`]);
      run(['edit', orphan, '--reason', 'past due', '--set', 'due=2020-01-01']);
      add(scope, ['question', 'add', `${scope} orphan question ${i}`, '--decider', `${scope} decider`, '--options', 'yes', '--options', 'no']);
      const closedCheck = add(scope, ['criterion', 'add', `${scope} withdrawn ${i}`]);
      const closed = add(scope, ['need', 'add', `${scope} closed ${i}`, '--targets', closedCheck]);
      run(['need', 'close', closed, '--by', 'fact', '--evidence', 'no longer needed']);
    }
  }
  for (const scope of ['a', 'b']) {
    const mine = groups[scope], other = groups[scope === 'a' ? 'b' : 'a'];
    run(['link', mine.direct, 'waits-on', other.root]);
    run(['link', mine.chain, 'depends-on', other.middle]);
    run(['link', mine.middle, 'waits-on', mine.root]);
  }
}

test.beforeAll(async () => {
  gy = await start({ build });
  all = JSON.parse(gy.gy(['list']));
  const statuses = new Map(all.map(row => [row.id, row.status]));
  for (const scope of ['a', 'b']) {
    const rows: Row[] = JSON.parse(gy.gy(['list', '--scope', scope]));
    const open = rows.filter(row => row.kind === 'Need' && row.status === 'open');
    // The CLI's own edges and derived statuses determine which open needs wait.
    const blocked = open.filter(row => JSON.parse(gy.gy(['show', row.id]))[0].edges.some(
      (edge: { name: string; to: string }) =>
        (edge.name === 'waits-on' || edge.name === 'depends-on') &&
        ['open', 'filed', 'approved'].includes(statuses.get(edge.to) || ''),
    ));
    cli[scope] = { rows, counts: {
      open_needs: open.length,
      blocked_needs: blocked.length,
      open_questions: rows.filter(row => row.kind === 'Question' && row.status === 'open').length,
      unmet_criteria: rows.filter(row => row.kind === 'Criterion' && row.status === 'unsatisfied').length,
      closed_needs: rows.filter(row => row.kind === 'Need' && row.status !== 'open').length,
    } };
  }
});
test.afterAll(async () => { await gy?.stop(); });

const main = (page: Page) => page.getByTestId('main');
const value = (page: Page, key: string) => main(page).locator(`[data-testid="stat"][data-key="${key}"]`).getByTestId('statValue');
const pick = (page: Page, scope: string) => page.getByTestId('scopes').locator(`button[data-s="${scope}"]`).click();
const ids = (rows: Row[]) => rows.map(row => row.id).sort();

async function matches(page: Page, request: APIRequestContext, scope: string) {
  const view = await (await request.get(`${gy.url}api/overview?scope=${scope}`)).json();
  expect(view.counts).toEqual(cli[scope].counts);
  for (const [key, api] of Object.entries({ open: 'open_needs', blocked: 'blocked_needs', questions: 'open_questions', unmet: 'unmet_criteria', done: 'closed_needs' })) {
    await expect(value(page, key)).toHaveText(String(cli[scope].counts[api]));
  }
  const open = cli[scope].rows.filter(row => row.kind === 'Need' && row.status === 'open');
  expect(ids(view.needs)).toEqual(ids(open));
  await expect(main(page).getByTestId('needCard')).toHaveCount(open.length);
  await expect(main(page).locator('[data-testid="needCard"][data-blocked="true"]')).toHaveCount(cli[scope].counts.blocked_needs);
  await expect(main(page).getByTestId('boardScopeName')).toHaveText([scope]);
  await expect(main(page).getByTestId('chain')).toHaveCount(view.blockers.length);
  await expect(main(page).getByTestId('overdue')).toHaveCount(view.overdue.length);
  await expect(main(page).getByTestId('unheld')).toHaveCount(view.unheld.length);
  for (const key of ['needs', 'overdue', 'unheld']) {
    expect(view[key].every((row: Row) => row.scope === scope)).toBe(true);
  }
  expect(ids(view.overdue)).toEqual(ids(cli[scope].rows.filter(row => row.kind === 'Criterion' && row.status === 'unsatisfied')));
  expect(ids(view.unheld)).toEqual(ids(cli[scope].rows.filter(row => row.unwaited || row.title.includes('orphan check'))));
  expect(view.deciders).toEqual([{ name: `${scope} decider`, count: cli[scope].counts.open_questions }]);
  await expect(main(page).getByTestId('deciders')).toContainText(`${scope} decider${cli[scope].counts.open_questions}`);

  const other = scope === 'a' ? 'b' : 'a';
  const mine = groups[scope], foreign = groups[other];
  for (const id of [mine.direct, mine.chain]) {
    const link = main(page).getByTestId('needCard').filter({ hasText: id });
    await expect(link).toHaveAttribute('data-blocked', 'true');
    await expect(link.getByTestId('scopeBadge')).toHaveText(other);
  }
  const root = main(page).getByTestId('chain').filter({ has: page.getByTestId('blockerRoot').getByRole('link', { name: new RegExp(`^${foreign.root} `) }) });
  await expect(root.getByTestId('blockerRoot').getByTestId('scopeBadge')).toHaveText(other);
  await expect(root.getByTestId('blockerVia').getByTestId('scopeBadge')).toHaveText(other);
  await expect(root.getByTestId('blockerVia').getByRole('link', { name: foreign.middle, exact: true })).toBeVisible();
  const allowed = new Set([foreign.root, foreign.middle]);
  for (const node of all.filter(row => row.scope === other && !allowed.has(row.id))) {
    await expect(main(page)).not.toContainText(node.title);
    await expect(main(page).getByRole('link', { name: new RegExp(node.id) })).toHaveCount(0);
  }
  const total = await (await request.get(`${gy.url}api/overview`)).json();
  expect(view.weeks.map(w => w.from)).toEqual(total.weeks.map(w => w.from));
  expect(view.weeks.reduce((sum, w) => sum + w.opened, 0)).toBe(cli[scope].rows.filter(row => row.kind === 'Need').length);
  expect(view.weeks.reduce((sum, w) => sum + w.done, 0)).toBe(cli[scope].counts.closed_needs);
  await expect(main(page).getByTestId('week').locator('title')).toHaveText(view.weeks.map(w => `${Number(w.from.slice(5, 7))}/${Number(w.from.slice(8))}: ${w.opened} / ${w.done}`));
  expect(view.scopes).toEqual(total.scopes.filter(row => row.name === scope));
  await expect(main(page).getByTestId('scopePulse').locator('rect')).toHaveCount(view.scopes[0].pulse.length);
  // The crossing's own scope and dates keep their meaning; only is separate.
  await expect(main(page).getByTestId('scopeCross')).toHaveAttribute('href', new RegExp(`^#/cross\\?from=.*&to=.*&scope=${scope}&only=${scope}$`));
}

for (const scope of ['a', 'b']) {
  test(`progress counts and rows follow ${scope}, with tagged foreign waits and via`, async ({ page, request }) => {
    await page.goto(`${gy.url}?lang=en#/?only=${scope}`);
    await matches(page, request, scope);
  });
}

test('scope choices and back/forward refresh progress without reloading', async ({ page, request }) => {
  await page.goto(`${gy.url}?lang=en#/`);
  await pick(page, 'a');
  await matches(page, request, 'a');
  await page.evaluate(() => { window['progressMarker'] = 'same document'; });
  await pick(page, 'b');
  await matches(page, request, 'b');
  await page.goBack();
  await expect(page).toHaveURL(/#\/\?only=a$/);
  await matches(page, request, 'a');
  await page.goForward();
  await expect(page).toHaveURL(/#\/\?only=b$/);
  await matches(page, request, 'b');
  expect(await page.evaluate(() => window['progressMarker'])).toBe('same document');
});

test('a late answer for the previous scope cannot overwrite progress', async ({ page, request }) => {
  let release!: () => void;
  const gate = new Promise<void>(resolve => { release = resolve; });
  let arrived!: () => void;
  const asked = new Promise<void>(resolve => { arrived = resolve; });
  await page.route('**/api/overview?scope=a', async route => {
    const response = await route.fetch();
    arrived();
    await gate;
    await route.fulfill({ response });
  });
  await page.goto(`${gy.url}?lang=en#/?only=a`);
  await asked;
  await pick(page, 'b');
  await matches(page, request, 'b');
  const answered = page.waitForResponse('**/api/overview?scope=a');
  release();
  await answered;
  await matches(page, request, 'b');
});

test('an unknown scope warns, shows zero counts, and can return to all', async ({ page, request }) => {
  await page.goto(`${gy.url}?lang=en#/overview?only=missing`);
  const warning = page.getByTestId('unknownScope');
  await expect(warning).toBeVisible();
  await expect(warning).toContainText('missing');
  for (const key of ['open', 'blocked', 'questions', 'unmet', 'done']) await expect(value(page, key)).toHaveText('0');
  await expect(main(page).getByTestId('needCard')).toHaveCount(0);
  await warning.getByRole('button').click();
  const total = await (await request.get(`${gy.url}api/overview`)).json();
  await expect(value(page, 'open')).toHaveText(String(total.counts.open_needs));
  await expect(warning).toHaveCount(0);
});

test('without only both progress URLs still show every scope', async ({ page, request }) => {
  const total = await (await request.get(`${gy.url}api/overview`)).json();
  expect(await (await request.get(`${gy.url}api/overview?scope=all`)).json()).toEqual(total);
  for (const hash of ['#/', '#/overview']) {
    await page.goto(`${gy.url}?lang=en${hash}`);
    await expect(value(page, 'open')).toHaveText(String(total.counts.open_needs));
    await expect(main(page).getByTestId('needCard')).toHaveCount(total.needs.length);
    await expect(main(page).getByTestId('boardScopeName')).toHaveText(['a', 'b']);
  }
});

test('a scope with only closed needs keeps its board row and pulse', async ({ page }) => {
  const closed = await start({ build: run => {
    const ac = JSON.parse(run(['--scope', 'b', 'criterion', 'add', 'closed check'])).id;
    const need = JSON.parse(run(['--scope', 'b', 'need', 'add', 'closed need', '--targets', ac])).id;
    run(['need', 'close', need, '--by', 'fact', '--evidence', 'no longer needed']);
  } });
  try {
    await page.goto(`${closed.url}?lang=en#/?only=b`);
    await expect(value(page, 'open')).toHaveText('0');
    await expect(value(page, 'done')).toHaveText('1');
    await expect(main(page).getByTestId('boardScopeName')).toHaveText('b');
    await expect(main(page).getByTestId('scopePulse')).toBeVisible();
  } finally { await closed.stop(); }
});

test('a scoped overview still reads the selected historical point', async ({ request }) => {
  const current = await (await request.get(`${gy.url}api/overview?scope=a`)).json();
  const then = await (await request.get(`${gy.url}api/overview?scope=a&at=1`)).json();
  expect(then.seq).toBe(1);
  expect(then.counts.open_needs).toBe(0);
  expect(then.counts.unmet_criteria).toBe(1);
  expect(then.needs).toEqual([]);
  expect(then.seq).toBeLessThan(current.seq);
});
