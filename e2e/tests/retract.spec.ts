import { expect, test } from '@playwright/test';
import { mkdtempSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { start, type Ledger } from '../fixtures/ledger';

/* n-0c6f: the node page must show where a passage was retracted, not only the
   relation's name on the map. A ledger of its own, so the shared fixture's
   counts stay what the other scenes read. */
let gy: Ledger;

test.beforeAll(async () => {
  gy = await start({
    build: run => {
      const file = join(mkdtempSync(join(tmpdir(), 'gy-retract-')), 'body.md');
      writeFileSync(file, '## Decision\nThe old passage stands here.\n');
      const older = JSON.parse(
        run(['--scope', 'a', 'decide', 'the retracted decision', '--scope-note', 'the old scope', '--body-file', file]),
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

test('the node page shows where a passage was retracted', async ({ page, request }) => {
  const decisions = (await (await request.get(`${gy.url}api/list?kind=Decision`)).json()).rows;
  const older = decisions.find((row: { title: string }) => row.title.startsWith('the retracted'));
  const node = await (await request.get(`${gy.url}api/node/${older.id}`)).json();

  // The answer keeps the record raw and carries the marked copy beside it.
  expect(node.body).not.toContain('[[');
  expect(node.body_marked).toContain('[[retracted by');

  await page.goto(`${gy.url}#/n/${older.id}`);
  const main = page.getByTestId('main');
  await expect(main.locator('pre')).toContainText('[[retracted by');
  // The heading names the retracting decision; the map only carries the
  // relation's name, so the id here is what tells the two apart.
  await expect(main.getByText(/narrowed by d-/)).toBeVisible();
});