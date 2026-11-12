import { expect, test } from '@playwright/test';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;
let older: string;
let newer: string;
const marks = ['Context passage alpha.', 'Decision passage beta.', 'Scope passage gamma.'];

test.beforeAll(async () => {
  gy = await start({ build: run => {
    const id = (args: string[]) => JSON.parse(run(args)).id as string;
    const body = join(mkdtempSync(join(tmpdir(), 'gy-narrows-')), 'body.md');
    writeFileSync(body, `## Context\n${marks[0]}\n\n## Decision\n${marks[1]}\n`);
    const ac = id(['--scope', 'a', 'criterion', 'add', 'purpose']);
    older = id(['--scope', 'a', 'decide', 'older', '--scope-note', marks[2], '--body-file', body, '--serves', ac]);
    newer = id(['--scope', 'a', 'decide', 'newer', '--scope-note', 'here', '--relate', 'narrows', older, '--mark', marks[0]]);
    for (const mark of marks.slice(1)) run(['link', newer, 'narrows', older, '--mark', mark]);
  } });
});
test.afterAll(async () => { await gy?.stop(); });

test('three passages appear in the node and both directions of its thread', async ({ page, request }) => {
  const node = await (await request.get(`${gy.url}api/node/${older}`)).json();
  expect(node.cancellation.narrowed).toEqual(marks.map(mark => ({ by: newer, mark })));
  await page.goto(`${gy.url}?lang=en#/n/${older}`);
  const main = page.getByTestId('main');
  for (const mark of marks.slice(0, 2)) {
    await expect(main.locator('pre')).toContainText(`[[retracted by ${newer}: ${mark}]]`);
  }
  await expect(main.getByTestId('facts')).toContainText(`[[retracted by ${newer}: ${marks[2]}]]`);
  for (const mark of marks) await expect(main.getByTestId('facts')).toContainText(mark);
  for (const [center, side, peer] of [[older, 'down', newer], [newer, 'up', older]]) {
    const path = await (await request.get(`${gy.url}api/node/${center}/path`)).json();
    expect(path[side][0].nodes.filter((n: { id: string }) => n.id === peer)).toHaveLength(1);
    expect(path[side][0].nodes.find((n: { id: string }) => n.id === peer).mark).toBe(marks.join('\n'));
    await page.goto(`${gy.url}?lang=en#/n/${center}`);
    await expect(main.locator('svg title').filter({ hasText: marks.join('\n') })).toHaveCount(1);
  }
});
