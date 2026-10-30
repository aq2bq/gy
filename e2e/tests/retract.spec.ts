import { expect, test } from '@playwright/test';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { start, type Ledger } from '../fixtures/ledger';

/* n-0c6f: the detail screen shows where a passage was retracted, not only the
   relation's name on the thread. A ledger of its own, so the shared fixture's
   counts stay what the other scenes read. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start({
    build: run => {
      const file = join(mkdtempSync(join(tmpdir(), 'gy-retract-')), 'body.md');
      writeFileSync(file, '## Decision\nThe old passage stands here.\n');
      const measure = JSON.parse(run(['--scope', 'a', 'criterion', 'add', 'measures one'])).id as string;
      const older = JSON.parse(
        run(['--scope', 'a', 'decide', 'the retracted decision', '--scope-note', 'the old scope', '--body-file', file, '--serves', measure]),
      ).id as string;
      run([
        '--scope', 'a', 'decide', 'the retracting decision', '--scope-note', 'a new call',
        '--relate', 'narrows', older, '--mark', 'The old passage stands here.',
      ]);
    },
  });
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the detail screen shows where a passage was retracted', async ({ page, request }) => {
  const nodes = (await (await request.get(`${gy.url}api/cross?q=retracted`)).json()).nodes;
  const older = nodes.find((row: { kind: string; title: string }) => row.kind === 'Decision' && row.title.startsWith('the retracted'));
  const node = await (await request.get(`${gy.url}api/node/${older.id}`)).json();

  expect(node.body).not.toContain('[[');
  expect(node.body_marked).toContain('[[retracted by');

  await page.goto(`${gy.url}?lang=en#/n/${older.id}`);
  const main = page.getByTestId('main');
  await expect(main.locator('pre')).toContainText('[[retracted by');
  await expect(main.getByTestId('facts')).toContainText(/narrowed by d-/);
});
