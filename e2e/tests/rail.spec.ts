import { expect, test, type Locator } from '@playwright/test';
import { start, type Ledger } from '../fixtures/ledger';

let gy: Ledger;

test.beforeAll(async () => {
  gy = await start();
});

test.afterAll(async () => {
  await gy?.stop();
});

test('the rail shows the ten newest writes, and only on a wide screen', async ({ page, request }) => {
  const answer = await (await request.get(`${gy.url}api/history?limit=10`)).json();
  await page.setViewportSize({ width: 1600, height: 900 });
  await page.goto(gy.url);

  const rows = page.getByTestId('railBody').getByRole('listitem');
  await expect(rows).toHaveCount(10);
  await expect(rows.first()).toContainText(`· ${answer.rows[0].seq}`);
  await expect(page.getByTestId('rail').getByRole('link', { name: /See all/ })).toHaveCount(1);

  await page.setViewportSize({ width: 1152, height: 720 });
  await expect(page.getByTestId('rail')).toBeHidden();
  await expect(rows).toHaveCount(0);

  // Dragging the window back follows: the rows return.
  await page.setViewportSize({ width: 1600, height: 900 });
  await expect(rows).toHaveCount(10);
});

test('the four entries line up their marks and their labels', async ({ page }) => {
  await page.goto(gy.url);
  const marks = page.getByTestId('navMark');
  const labels = page.getByTestId('navLabel');
  await expect(marks).toHaveCount(4);
  await expect(labels).toHaveCount(4);
  const box = async (loc: Locator) => {
    const found = await loc.boundingBox();
    if (!found) throw new Error('the element has no box');
    return found;
  };

  const mark = await box(marks.first());
  const label = await box(labels.first());
  for (let at = 1; at < 4; at++) {
    expect(Math.abs((await box(marks.nth(at))).x - mark.x)).toBeLessThan(1);
    expect(Math.abs((await box(labels.nth(at))).x - label.x)).toBeLessThan(1);
    // The same size, measured at the box: the glyphs keep their own widths.
    expect(Math.abs((await box(marks.nth(at))).width - mark.width)).toBeLessThan(1);
  }

  // The search's mark is for the eye only; the other three keep their names.
  await expect(page.getByTestId('searchEntry').getByTestId('navMark')).toHaveAttribute('aria-hidden', 'true');
  await expect(marks.first()).not.toHaveAttribute('aria-hidden', 'true');
});

test('the search entry lives in the sidebar, at every width', async ({ page }) => {
  for (const width of [1600, 1152, 900]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto(gy.url);
    const entry = page.getByTestId('searchEntry');
    await expect(entry).toBeVisible();
    // The dead frames are gone from both the rail and the top bar.
    await expect(page.getByTestId('search')).toHaveCount(0);
    await expect(page.getByTestId('rail').getByTestId('search')).toHaveCount(0);
    await expect(page.getByTestId('topbar').getByTestId('search')).toHaveCount(0);
    await expect(entry).toContainText('⌘K');
    await expect(entry).toContainText('/');

    await entry.click();
    await expect(page.getByTestId('palette')).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(page.getByTestId('palette')).toBeHidden();
  }
});

test('the search entry outlives the panel redraws a scope switch causes', async ({ page, request }) => {
  const errors: Error[] = [];
  page.on('pageerror', error => errors.push(error));

  await page.setViewportSize({ width: 1600, height: 900 });
  await page.goto(gy.url);
  const entry = page.getByTestId('searchEntry');
  await expect(entry).toHaveCount(1);

  // Two switches: the nav is rewritten each time, and the fault used to show
  // only on the second one (n-1d12).
  const shell = await (await request.get(`${gy.url}api/shell`)).json();
  const names: string[] = shell.scopes.map((item: { name: string }) => item.name);
  const scopes = page.getByTestId('scopes');
  await scopes.getByRole('button', { name: new RegExp(`^${names[0]}`) }).click();
  await expect(entry).toHaveCount(1);
  await scopes.getByRole('button', { name: new RegExp(`^${names[1]}`) }).click();
  await expect(entry).toHaveCount(1);
  await entry.click();
  await expect(page.getByTestId('palette')).toBeVisible();
  await expect(page.getByTestId('rail').getByText(/Live/)).toHaveCount(1);

  expect(errors).toEqual([]);
});