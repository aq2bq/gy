import {test, expect} from '@playwright/test';
import {mouse, open, enter} from '../helpers.mjs';

test.beforeEach(async ({page}) => {
  const errors = [];
  page.on('pageerror', e => errors.push(e.message));
  page._detailErrors = errors;
  const requests = [];
  page.on('request', r => { if (/^https?:/.test(r.url())) requests.push(r.url()); });
  await page.route('https://**/*', route => route.abort());
  await page.route('http://**/*', route => route.abort());
  page._detailRequests = requests;
  await open(page, 'reading');
});
test.afterEach(async ({page}) => {
  expect(page._detailErrors).toEqual([]);
  // URL-bearing data and links are allowed; loading/drawing must never fetch them.
  expect(page._detailRequests).toEqual([]);
});

// Count actually wrapped characters; final/short lines are excluded by the test.
async function lines(locator) {
  return locator.evaluate(el => {
    const walker = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
    const result = new Map();
    for (let node; (node = walker.nextNode());) {
      let offset = 0;
      for (const char of node.textContent) {
        const range = document.createRange();
        range.setStart(node, offset); offset += char.length; range.setEnd(node, offset);
        const r = range.getBoundingClientRect();
        if (r.width > 0 && r.height > 0) {
          const key = Math.round(r.top);
          result.set(key, (result.get(key) || '') + char);
        }
      }
    }
    return [...result.values()];
  });
}

for (const width of [1280,1440,1920,2560]) test(`detail and graph have equal width at ${width}px`, async ({page}, info) => {
  await page.setViewportSize({width,height:1000});
  await enter(page, 'D-501');
  const sizes = await page.evaluate(() => {
    const g=document.querySelector('#graphPane').getBoundingClientRect(), d=document.querySelector('#detail').getBoundingClientRect();
    return {graph:g.width,detail:d.width,ratio:d.width/g.width};
  });
  expect(sizes.ratio).toBeGreaterThanOrEqual(.9);
  expect(sizes.ratio).toBeLessThanOrEqual(1.1);
  await expect(page.locator('[data-node-id="D-501"]')).toBeInViewport();
  await info.attach('width.json',{body:JSON.stringify(sizes),contentType:'application/json'});
});

for (const width of [1400,1920,2560]) test(`reading measure for Japanese and Latin at ${width}px`, async ({page},info) => {
  await page.setViewportSize({width,height:1000});
  const result=[];
  for (const [id, lower, upper] of [['D-501',30,45],['D-502',45,90]]) {
    await enter(page,id);
    await page.evaluate(() => document.fonts.ready);
    for (const selector of ['#detailScope .detail-prose p','#body > p:first-child']) {
      const paragraph=page.locator(selector);
      const wrapped=await lines(paragraph);
      const full=wrapped.slice(0,-1);
      expect(full.length).toBeGreaterThan(3);
      const counts=full.map(line=>[...line.trimEnd()].length);
      expect(Math.min(...counts)).toBeGreaterThanOrEqual(lower);
      expect(Math.max(...counts)).toBeLessThanOrEqual(upper);
      expect(await paragraph.evaluate(el=>parseFloat(getComputedStyle(el).fontSize))).toBe(16);
      expect(await paragraph.evaluate(el=>!!el.closest('#detailDeclarations'))).toBe(false);
      result.push({id,selector,counts});
    }
  }
  await info.attach('line-lengths.json',{body:JSON.stringify(result),contentType:'application/json'});
  await page.screenshot({path:info.outputPath('reading.png')});
});

test('badges preserve closed values, status qualifiers and unknown states', async ({page}) => {
  for (const [id,kind] of [['D-501','decision'],['N-501','need'],['G-501','gate'],['Q-504','question'],['AC-501','criterion'],['#501','requirement']]) {
    await enter(page,id);
    await expect(page.locator('.detail-badge[data-field="type"]')).toHaveText('Type '+kind);
    await expect(page.locator('.detail-badge[data-field="scope"]')).toHaveText('Scope reading');
  }
  const states=['unfiled','defining','awaiting-design','awaiting-approval','awaiting-implementation','awaiting-audit','awaiting-pr','awaiting-merge','awaiting-production','awaiting-cleanup','complete','future-state'];
  for (const [index,state] of states.entries()) {
    await enter(page,'#'+(501+index));
    await expect(page.locator('.detail-badge[data-field="status"]')).toHaveText('Status '+state);
    if (state==='awaiting-approval') await expect(page.locator('.status-note')).toHaveText('awaiting-approval (owner review)');
    await expect(page.locator('#detailAdditional [data-key="status"]')).toHaveCount(0);
  }
  for (const [index,method] of ['fact','decision','non-decision'].entries()) {
    await enter(page,'Q-'+(501+index));
    await expect(page.locator('.detail-badge[data-field="status"]')).toHaveText('Status closed');
    await expect(page.locator('.detail-badge[data-field="closed_by"]')).toHaveText('Closed by '+method);
  }
  for (const [id,value] of [['AC-501','Satisfied'],['AC-502','Not satisfied']]) {
    await enter(page,id);
    await expect(page.locator('.detail-badge[data-field="satisfied"]')).toHaveText('Acceptance '+value);
    await expect(page.locator('#detailAdditional [data-key="satisfied"]')).toHaveCount(0);
  }
});

test('unknown attributes retain full values and unsafe text is inert', async ({page}) => {
  await enter(page,'D-501');
  const expected = await page.evaluate(() => window.GY_DATA.nodes.find(n=>n.id==='D-501').attrs);
  for (const key of ['custom_false','custom_zero','custom_null','custom_nested','custom_long','custom_url','supersedes']) {
    const pre=page.locator(`#detailAdditional [data-key="${key}"] pre`);
    const raw=await pre.textContent();
    if (typeof expected[key]==='string') expect(raw).toBe(expected[key]);
    else expect(JSON.parse(raw)).toEqual(expected[key]);
  }
  await expect(page.locator('#detailDeclarations [data-field="pr_url"] a')).toHaveAttribute('href','https://example.test/pr/42');
  await expect(page.locator('#detail img, #detail a[href^="javascript:"]')).toHaveCount(0);
  const long=page.locator('#detailAdditional [data-key="custom_long"] pre');
  await long.evaluate(el=>el.scrollIntoView({block:'end'}));
  const end = await long.evaluate(el=>{
    const r=document.createRange(),n=el.firstChild;r.setStart(n,n.length-4);r.setEnd(n,n.length);
    const end=r.getBoundingClientRect(),panel=document.querySelector('#detail').getBoundingClientRect();
    return end.bottom<=panel.bottom && end.top>=panel.top && end.right<=panel.right;
  });
  expect(end).toBe(true);
});

test('core mark projection distinguishes found and missing passages without moving marks', async ({page}) => {
  await enter(page,'D-502');
  await expect(page.locator('#body')).toContainText('This decision applies to the recorded conditions. ⟦Superseded: D-501⟧');
  await expect(page.locator('#detailMarks [data-found="true"]')).toContainText('Found in the source body.');
  await enter(page,'D-503');
  const projected = await page.evaluate(()=>window.GY_DATA.nodes.find(n=>n.id==='D-503').superseded_by);
  expect(projected).toEqual(['D-504']);
  expect(await page.evaluate(()=>window.GY_DATA.edges.some(e=>e.source==='D-504' && e.target==='D-503'))).toBe(false);
  await expect(page.locator('.detail-badge[data-field="superseded"]')).toHaveText('Decision Superseded');
  await expect(page.locator('#detailMarks [data-found="false"]')).toContainText('A passage absent from this body');
  await expect(page.locator('#detailMarks')).toContainText('Location in body could not be found.');
  await expect(page.locator('#body')).not.toContainText('⟦Superseded');
});

test('related node selection preserves layout until explicit focus', async ({page}) => {
  await enter(page,'D-501');
  await page.evaluate(()=>{
    window.fitChanges=0;
    new MutationObserver(ms=>window.fitChanges+=ms.filter(m=>m.attributeName==='transform').length).observe(document.querySelector('#svg > g'),{attributes:true});
  });
  await mouse(page, page.locator('.detail-relations').filter({hasText:'Decision lineage'}).locator('[data-go="D-502"]'));
  // D-39 replaces click-to-focus with selection only.
  await expect(page.locator('#focusStatus')).toContainText('Focus: D-501 ·');
  await expect(page.locator('.detail-id')).toHaveText('D-502');
  expect(await page.evaluate(()=>window.fitChanges)).toBe(0);
  await mouse(page, '#detailFocus');
  expect(await page.evaluate(()=>window.fitChanges)).toBe(1);
  await mouse(page,'#closeDetail');
  await expect(page.locator('#focusStatus')).toContainText('Focus: D-502 ·');
});
