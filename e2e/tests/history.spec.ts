import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';
import { join, resolve } from 'node:path';
import { start, type Ledger } from '../fixtures/ledger';

/** The release binary the fixture also starts. */
const GY = resolve(__dirname, '..', '..', 'target', 'release', 'gy');

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
  // A second writer, so the actor filter has something to narrow to.
  execFileSync(
    GY,
    ['-C', gy.dir, '--scope', 'a', 'criterion', 'add', 'a second opinion', '--json'],
    {
      env: { ...process.env, GY_ACTOR: 'reviewer', XDG_DATA_HOME: join(gy.dir, 'data') },
      encoding: 'utf8',
    },
  );
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the history page matches /api/history and filters by actor', async ({ page, request }) => {
  const answer = await (await request.get(`${gy.url}api/history`)).json();
  await page.goto(`${gy.url}#/history`);

  await expect(page.locator('.hi')).toHaveCount(answer.rows.length);
  await expect(page.locator('.hi .when').first()).toContainText(`· ${answer.rows[0].seq}`);

  await page.locator('#hist-actors button[data-a="reviewer"]').click();
  const filtered = await (await request.get(`${gy.url}api/history?actor=reviewer`)).json();
  expect(filtered.rows.length).toBeLessThan(answer.rows.length);
  await expect(page.locator('.hi')).toHaveCount(filtered.rows.length);
});