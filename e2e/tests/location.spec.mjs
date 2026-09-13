import {test, expect} from '@playwright/test';
import {fixture, mouse, enter} from '../helpers.mjs';

test.beforeEach(async ({page, context}) => {
  page._errors = []; page._requests = [];
  context.on('page', p => p.on('pageerror', e => page._errors.push(e.message)));
  page.on('pageerror', e => page._errors.push(e.message));
  context.on('request', r => { if (/^https?:/.test(r.url())) page._requests.push(r.url()); });
});
test.afterEach(async ({page}) => { expect(page._errors).toEqual([]); expect(page._requests).toEqual([]); });

test('URL selects IDs, reports missing IDs, and restores state through history', async ({page, context}) => {
  await page.goto(fixture('large') + '#D-1');
  await expect(page.locator('.detail-id')).toHaveText('D-1');
  await enter(page, 'D-2');
  await expect.poll(() => decodeURIComponent(page.url())).toContain('"selected":"D-2"');
  const depth = await page.evaluate(() => history.length);
  await page.locator('[data-node-id="D-3"]').press('Enter');
  await expect.poll(() => decodeURIComponent(page.url())).toContain('"selected":"D-3"');
  expect(await page.evaluate(() => history.length)).toBe(depth + 1);
  await page.goBack();
  await expect(page.locator('.detail-id')).toHaveText('D-2');
  await page.locator('#q').fill('decision');
  await page.locator('#scopeSel').selectOption('s0');
  await mouse(page, '#genealogy');
  await mouse(page, '[data-tab="jams"]');
  await expect.poll(() => decodeURIComponent(page.url())).toContain('"tab":"jams"');
  const url = page.url();
  const copy = await context.newPage();
  await copy.goto(url);
  await expect(copy.locator('.detail-id')).toHaveText('D-2');
  await expect(copy.locator('#q')).toHaveValue('decision');
  await expect(copy.locator('#scopeSel')).toHaveValue('s0');
  await expect(copy.locator('#genealogy')).toHaveAttribute('aria-pressed', 'true');
  await copy.close();
  await page.reload();
  await expect(page.locator('.detail-id')).toHaveText('D-2');
  await expect(page.locator('#focusStatus')).toContainText('Focus: D-2');
  await expect(page.locator('#q')).toHaveValue('decision');
  await expect(page.locator('#scopeSel')).toHaveValue('s0');
  await expect(page.locator('#genealogy')).toHaveAttribute('aria-pressed', 'true');
  await expect(page.locator('[data-tab="jams"]')).toHaveClass('on');
  await mouse(page, '[data-tab="overview"]');
  await page.goBack();
  expect(page.url()).toBe(url);
  await expect(page.locator('[data-tab="jams"]')).toHaveClass('on');
  await page.goForward();
  await expect(page.locator('[data-tab="overview"]')).toHaveClass('on');
  const beforeZoom = page.url();
  const filterUrl = page.url();
  await mouse(page, '#zin');
  expect(page.url()).toBe(filterUrl);
  expect(filterUrl).toBe(beforeZoom);
  await page.goto(fixture('large') + '#D-999999');
  await expect(page.locator('#locationStatus')).toContainText('Node not found: D-999999');
  await page.goto(fixture('reading') + '#D-501');
  await page.locator('#stateSel').selectOption({index:1});
  const state = await page.locator('#stateSel').inputValue();
  expect(state).not.toBe('');
  await page.locator('#qstatus').selectOption('closed');
  await page.locator('#criterion').selectOption('yes');
  await mouse(page, '[data-kind="question"]');
  await expect.poll(() => decodeURIComponent(page.url())).toContain('"question":false');
  await page.reload();
  await expect(page.locator('#stateSel')).toHaveValue(state);
  await expect(page.locator('#qstatus')).toHaveValue('closed');
  await expect(page.locator('#criterion')).toHaveValue('yes');
  await expect(page.locator('[data-kind="question"]')).not.toHaveClass(/on/);


});

test('node selection preserves placement, focus and filters until explicit focus', async ({page}) => {
  await page.goto(fixture('large'));
  await enter(page, 'D-1');
  await mouse(page, '#closeDetail');
  const placement = () => page.locator('#svg').evaluate(svg => ({
    transform: svg.querySelector(':scope > g').getAttribute('transform'),
    nodes: [...svg.querySelectorAll('[data-node-id]')].map(n => ({id:n.dataset.nodeId, transform:n.getAttribute('transform')}))
  }));
  const before = await placement();
  await mouse(page, '[data-node-id="D-2"] > path');
  await expect(page.locator('.detail-id')).toHaveText('D-2');
  await expect(page.locator('#focusStatus')).toContainText('Focus: D-1 ·');
  expect(await placement()).toEqual(before);
  await expect(page.locator('#scopeSel')).toHaveValue('');
  await expect(page.locator('#q')).toHaveValue('');
  await expect(page.locator('#displayStatus')).toBeVisible();
  await expect(page.locator('#detailFocus')).toContainText('hops around D-2');
  await expect(page.locator('#detailFocus')).toContainText('nodes before filters');
  await mouse(page, '#detailFocus');
  await expect(page.locator('#focusStatus')).toContainText('Focus: D-2 ·');
  await mouse(page, '#focusBack');
  await expect(page.locator('#focusStatus')).toContainText('Focus: D-1 ·');
});

for (const width of [1280, 1440, 1920]) test(`permanent toolbar remains operable at ${width}px`, async ({page}, info) => {
  await page.setViewportSize({width, height:1000});
  await page.goto(fixture('reading') + '#D-501');
  expect(await page.locator('#tabs button').allTextContents()).toEqual(['Overview', 'Progress', 'Blockers']);
  await expect(page.locator('[data-tab="filters"]')).toHaveCount(0);
  for (const tab of ['overview', 'progress', 'jams']) {
    await mouse(page, `[data-tab="${tab}"]`);
    for (const id of ['q','typeChips','scopeSel','stateSel','qstatus','criterion','clearFilter','genealogy','hopRadius'])
      await expect(page.locator('#'+id)).toBeVisible();
  }
  const controls = await page.locator('#filters button, #filters input, #filters select').evaluateAll(es => es.map(el => {
    const r = el.getBoundingClientRect(), hit = document.elementFromPoint(r.x+r.width/2, r.y+r.height/2);
    return {id:el.id || el.dataset.kind, left:r.left, right:r.right, top:r.top, bottom:r.bottom, hit:hit === el || el.contains(hit)};
  }));
  expect(controls.length).toBeGreaterThan(15);
  for (const c of controls) {
    expect(c.left, c.id).toBeGreaterThanOrEqual(0); expect(c.right, c.id).toBeLessThanOrEqual(width);
    expect(c.bottom, c.id).toBeLessThan(1000); expect(c.hit, c.id).toBe(true);
  }
  const toolbarTop = await page.locator('#filters').evaluate(el=>el.getBoundingClientRect().top);
  const headerBottom = await page.locator('header').evaluate(el=>el.getBoundingClientRect().bottom);
  expect(toolbarTop).toBeCloseTo(headerBottom, 1);
  await page.locator('#hopRadius').selectOption('2');
  await expect(page.locator('#detailFocus')).toContainText('2 hops');
  await mouse(page, '#detailFocus');
  await expect(page.locator('#focusStatus')).toContainText('2 hops');
  await page.reload();
  await expect(page.locator('#hopRadius')).toHaveValue('2');
  await expect(page.locator('#focusStatus')).toContainText('2 hops');
  await mouse(page, '[data-tab="overview"]');
  await mouse(page, page.locator('#stateBars .bar').first());
  await expect(page.locator('[data-tab="overview"]')).toHaveClass('on');
  expect(await page.locator('#stateSel').inputValue()).not.toBe('');
  await page.screenshot({path:info.outputPath('toolbar.png')});
  await info.attach('toolbar-controls', {body:JSON.stringify(controls), contentType:'application/json'});
});


test('viewport redraw preserves a pending neighborhood target', async ({page}) => {
  await page.goto(fixture('reading'));
  await page.locator('#hopFrom').fill('D-501');
  await page.setViewportSize({width:2560,height:1000});
  await expect(page.locator('#hopFrom')).toHaveValue('D-501');
  await expect(page.locator('#applyHop')).toContainText('around D-501');
  await mouse(page, '#applyHop');
  await expect(page.locator('#focusStatus')).toContainText('Focus: D-501 ·');
  await mouse(page, '#focusAll');
  await page.locator('#hopFrom').fill('D-502');
  await mouse(page, '#clearFilter');
  await expect(page.locator('#hopFrom')).toHaveValue('');
});
