import { expect, test } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the sidebar footer links to the gy repository', async ({ page }) => {
  await page.goto(gy.url);
  const repo = page.getByTestId('repo');

  // The destination is read from the attributes: CI cannot leave the machine.
  await expect(repo).toHaveAttribute('href', 'https://github.com/aq2bq/gy');
  await expect(repo).toHaveAttribute('target', '_blank');
  await expect(repo).toHaveAttribute('rel', 'noopener noreferrer');

  // It is the last child of the footer, below the clock.
  await expect(page.getByTestId('foot').locator('> *').last()).toHaveAttribute(
    'data-testid',
    'repo',
  );

  // A name in both languages, and no reload on the way (n-b380).
  await expect(repo).toHaveAttribute('aria-label', 'gy repository');
  await page.evaluate(() => {
    (window as unknown as { kept?: boolean }).kept = true;
  });
  await page.getByTestId('sidebar').getByRole('button', { name: '日本語' }).click();

  await expect(repo).toHaveAttribute('aria-label', 'gy のリポジトリ');
  await expect(repo).toHaveAttribute('title', 'gy のリポジトリ');
  expect(await page.evaluate(() => (window as unknown as { kept?: boolean }).kept)).toBe(true);
});