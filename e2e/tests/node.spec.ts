import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the decision page matches /api/node', async ({ page, request }) => {
  const decisions = (await (await request.get(`${gy.url}api/list?kind=Decision`)).json()).rows;
  const decision = decisions.find((row: { title: string }) => row.title.startsWith('closes'));
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();

  await page.goto(`${gy.url}#/n/${decision.id}`);
  await expect(page.locator('.nh h1')).toHaveText(node.title);
  await expect(page.locator('.scope')).toContainText(node.data.Decision.scope.text);
  await expect(page.locator('.hi')).toHaveCount(node.history.length);
  // Every edge is a box, plus the node itself in the middle.
  await expect(page.locator('.map .box')).toHaveCount(node.edges.length + 1);

  // A box opens the page of the node it points at.
  const peer = node.edges[0];
  await page.locator(`.map a[href="#/n/${peer.to}"]`).click();
  await expect(page.locator('.nh h1')).not.toHaveText(node.title);
});

test('the decision names the question it closed', async ({ page, request }) => {
  const decisions = (await (await request.get(`${gy.url}api/list?kind=Decision`)).json()).rows;
  const decision = decisions.find((row: { title: string }) => row.title.startsWith('closes'));
  const node = await (await request.get(`${gy.url}api/node/${decision.id}`)).json();
  const edge = node.edges.find((item: { name: string }) => item.name === 'closed-by');
  const question = await (await request.get(`${gy.url}api/node/${edge.to}`)).json();

  await page.goto(`${gy.url}#/n/${decision.id}`);
  const box = page.locator(`.map a[href="#/n/${edge.to}"]`);
  await expect(box).toHaveCount(1);
  // The relation reads as "closed" / "閉じた論点" from this side (the label
  // sits beside the box, not inside it).
  const labels = await page
    .locator('.map text.rel')
    .evaluateAll(nodes => nodes.map(node => node.textContent));
  expect(labels.join(' ')).toMatch(/closed|閉じた/);

  // The box opens the question's page.
  await box.click();
  await expect(page.locator('.nh h1')).toHaveText(question.title);
  await expect(page.locator('.hi')).toHaveCount(question.history.length);
});
