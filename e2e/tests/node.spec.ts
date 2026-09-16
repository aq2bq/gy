import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

/// The name a box on the map shows for an edge's peer.
const named = (edge: { alias?: string; to: string }) => edge.alias || edge.to;

test('the decision page matches /api/node', async ({ page, request }) => {
  const decisions = (await (await request.get(`${gy.url}api/list?kind=Decision`)).json()).rows;
  const decision = decisions.find((row: { title: string }) => row.title.startsWith('closes'));
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  const words = await (await request.get(`${gy.url}assets/i18n.json`)).json();

  await page.goto(`${gy.url}#/n/${decision.id}`);
  const main = page.getByTestId('main');
  await expect(main.getByRole('heading', { level: 1 })).toHaveText(node.title);
  await expect(main.getByText(node.data.Decision.scope.text)).toBeVisible();
  await expect(main.getByRole('list', { name: /History/ }).getByRole('listitem')).toHaveCount(node.history.length);
  // Every edge is a box, plus the node itself in the middle.
  await expect(main.locator('svg rect')).toHaveCount(node.edges.length + 1);

  // The body is open from the start, with no fold to open (n-d9a6).
  await expect(main.locator('summary')).toHaveCount(0);
  await expect(main.getByText(words.en.bodyTitle, { exact: true })).toBeVisible();
  await expect(main.locator('pre')).not.toBeEmpty();

  // A box opens the page of the node it points at.
  const peer = node.edges[0];
  await main.locator('svg a').filter({ hasText: named(peer) }).click();
  await expect(main.getByRole('heading', { level: 1 })).not.toHaveText(node.title);
});

test('the decision names the question it closed', async ({ page, request }) => {
  const decisions = (await (await request.get(`${gy.url}api/list?kind=Decision`)).json()).rows;
  const decision = decisions.find((row: { title: string }) => row.title.startsWith('closes'));
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  const edge = node.edges.find((item: { name: string }) => item.name === 'closed-by');
  const question = await (await request.get(`${gy.url}api/node/${edge.to}`)).json();
  const main = page.getByTestId('main');

  await page.goto(`${gy.url}#/n/${decision.id}`);
  const box = main.locator('svg a').filter({ hasText: named(edge) });
  await expect(box).toHaveCount(1);
  // The relation reads as "closed" / "閉じた論点" from this side (the label
  // sits beside the box, not inside it).
  const labels = await main
    .locator('svg text')
    .evaluateAll(nodes => nodes.map(node => node.textContent));
  expect(labels.join(' ')).toMatch(/closed|閉じた/);

  // The box opens the question's page.
  await box.click();
  await expect(main.getByRole('heading', { level: 1 })).toHaveText(question.title);
  await expect(main.getByRole('list', { name: /History/ }).getByRole('listitem')).toHaveCount(question.history.length);
});