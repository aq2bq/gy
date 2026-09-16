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
  const words: Record<string, RegExp> = { wait: /waiting/, next: /next/, resume: /in progress/ };

  await page.goto(gy.url);
  const eyes = page.getByTestId('eyes');
  for (const key of ['wait', 'next', 'resume']) {
    const link = eyes.getByRole('link', { name: words[key] });
    await expect(link).toContainText(String(counts[key]));
    await link.click();
    await expect(page).toHaveURL(new RegExp(`#/eye/${key}$`));
    await expect(page.getByTestId('main').getByRole('link')).toHaveCount(counts[key]);
  }

  // A row opens its node page.
  await page.getByTestId('main').getByRole('link').first().click();
  await expect(page).toHaveURL(/#\/n\//);

  // The scope narrows both the box and the rows.
  await page.goto(`${gy.url}#/eye/wait`);
  const scoped = await (await request.get(`${gy.url}api/now?scope=b`)).json();
  await page.getByTestId('scopes').getByRole('button', { name: /^b/ }).click();
  await expect(page.getByTestId('main').getByRole('link')).toHaveCount(scoped.waiting.length);
  await expect(page.getByTestId('eyes').getByRole('link', { name: /waiting/ })).toContainText(String(scoped.waiting.length));
});