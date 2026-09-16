import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('each sidebar box opens exactly the rows it counts', async ({ page, request }) => {
  const answer = await (await request.get(`${gy.url}api/now`)).json();
  const counts: Record<string, number> = {
    wait: answer.waiting.length,
    next: answer.ready.length,
    resume: answer.resume.in_progress.length,
  };

  await page.goto(gy.url);
  for (const key of ['wait', 'next', 'resume']) {
    const link = page.locator(`#eyes3 a[href="#/eye/${key}"]`);
    await expect(link.locator('b')).toHaveText(String(counts[key]));
    await link.click();
    await expect(page).toHaveURL(new RegExp(`#/eye/${key}$`));
    await expect(page.locator('#main .row.wide')).toHaveCount(counts[key]);
  }

  // A row opens its node page.
  await page.locator('#main .row.wide').first().click();
  await expect(page).toHaveURL(/#\/n\//);

  // The scope narrows both the box and the rows.
  await page.goto(`${gy.url}#/eye/wait`);
  const scoped = await (await request.get(`${gy.url}api/now?scope=b`)).json();
  await page.locator('#nav button[data-s="b"]').click();
  await expect(page.locator('#main .row.wide')).toHaveCount(scoped.waiting.length);
  await expect(page.locator('#eyes3 a[href="#/eye/wait"] b')).toHaveText(String(scoped.waiting.length));
});