import {test, expect} from '@playwright/test';
import {fixture, mouse} from '../helpers.mjs';

test.beforeEach(async ({page}) => {
  page._errors=[];page._requests=[];
  page.on('pageerror',e=>page._errors.push(e.message));
  page.on('request',r=>{if(/^https?:/.test(r.url()))page._requests.push(r.url());});
});
test.afterEach(async ({page})=>{expect(page._errors).toEqual([]);expect(page._requests).toEqual([]);});

test('table retains full titles, six columns, and a real whole-row link at 1440px', async ({page},info)=>{
  await page.setViewportSize({width:1440,height:1000});
  await page.goto(fixture('reading'));
  await expect(page.locator('#recordList th')).toHaveCount(6);
  await expect(page.locator('#recordList tbody tr')).toHaveCount(24);
  await expect(page.locator('#listCount')).toHaveText('24 results');
  await expect(page.locator('tr[data-record-id="#512"] td[data-column="status"]')).toHaveText('future-state');
  const title=page.locator('tr[data-record-id="D-501"] td[data-column="title"]');
  const full=await page.evaluate(()=>window.GY_DATA.nodes.find(n=>n.id==='D-501').title);
  await expect(title).toHaveText(full);
  const measure=await title.evaluate(el=>{
    const style=getComputedStyle(el), canvas=document.createElement('canvas'), ctx=canvas.getContext('2d');ctx.font=style.font;
    const width=el.getBoundingClientRect().width-parseFloat(style.paddingLeft)-parseFloat(style.paddingRight);
    const node=el.firstChild, lines=new Map();
    for(let i=0;i<node.length;i++){const r=document.createRange();r.setStart(node,i);r.setEnd(node,i+1);const y=r.getBoundingClientRect().top;lines.set(y,(lines.get(y)||0)+1);}
    return {width,minimum:ctx.measureText('漢'.repeat(30)).width,lines:[...lines.values()]};
  });
  expect(measure.width).toBeGreaterThanOrEqual(measure.minimum);
  expect(measure.lines.length).toBeGreaterThan(1);
  for(const n of measure.lines.slice(0,-1))expect(n).toBeGreaterThanOrEqual(30);
  const transform=await page.locator('#svg > g').getAttribute('transform');
  await mouse(page,title);
  await expect(page.locator('.detail-id')).toHaveText('D-501');
  await expect.poll(()=>decodeURIComponent(page.url())).toContain('"selected":"D-501"');
  expect(await page.locator('#svg > g').getAttribute('transform')).toBe(transform);
  await expect(page.locator('a[data-record="D-501"]')).toHaveAttribute('href',/#view=/);
  await expect(page.locator('#listPane')).toBeInViewport();await expect(page.locator('#svg')).toBeInViewport();
  await page.screenshot({path:info.outputPath('n22-1440.png')});
  await info.attach('title-measure',{body:JSON.stringify(measure),contentType:'application/json'});
});

test('type and lifecycle filters select the same records for list and graph',async({page})=>{
  await page.goto(fixture('reading'));
  await mouse(page,'[data-kind="decision"]');
  await expect(page.locator('#recordList tbody tr')).toHaveCount(4);
  await expect(page.locator('#listCount')).toHaveText('4 results');
  expect(await page.locator('#recordList tbody td[data-column="type"]').allTextContents()).toEqual(Array(4).fill('decision'));
  expect(await page.locator('#svg [data-node-id]').count()).toBe(4);
  await mouse(page,'[data-kind="question"]');
  await page.locator('#qstatus').selectOption('closed');
  await expect(page.locator('#recordList tbody tr')).toHaveCount(3);
  expect(await page.locator('#recordList td[data-column="status"]').allTextContents()).toEqual(['Closed','Closed','Closed']);
  await expect(page.locator('#listCount')).toHaveText('3 results');
  await mouse(page,'[data-kind="criterion"]');
  await page.locator('#criterion').selectOption('yes');
  await expect(page.locator('#recordList tbody tr')).toHaveCount(1);
  await expect(page.locator('#recordList td[data-column="status"]')).toHaveText('Satisfied');
  await page.locator('#q').fill('NO_MATCH');
  await expect(page.locator('#recordList tbody tr')).toHaveCount(0);
  await expect(page.locator('#listEmpty')).toBeVisible();
  await expect(page.locator('#listCount')).toHaveText('0 results');
});

test('column ordering restores on reload and browser back',async({page})=>{
  await page.goto(fixture('reading'));
  const rows=()=>page.locator('#recordList tbody a').allTextContents();
  const original=await rows();
  await mouse(page,'#recordList [data-sort="id"]');
  await expect(page.locator('th[data-column="id"]')).toHaveAttribute('aria-sort','descending');
  expect(await rows()).toEqual([...original].reverse());
  await expect.poll(()=>decodeURIComponent(page.url())).toContain('"direction":"desc"');
  await page.reload();
  expect(await rows()).toEqual([...original].reverse());
  await page.goBack();
  await expect(page.locator('th[data-column="id"]')).toHaveAttribute('aria-sort','ascending');
  expect(await rows()).toEqual(original);
});

test('a 1000-record list renders within the existing initial-load watchdog',async({page},info)=>{
  const start=performance.now();await page.goto(fixture('large'));
  await expect(page.locator('#recordList tbody tr')).toHaveCount(1000);
  const ms=performance.now()-start;
  expect(ms).toBeLessThan(10000);
  await expect(page.locator('#listCount')).toHaveText('1000 results');
  await info.attach('list-performance',{body:JSON.stringify({nodes:1000,milliseconds:ms}),contentType:'application/json'});
});

test('cluster keyboard activation applies its scope and type to the table',async({page})=>{
  await page.goto(fixture('large'));
  await page.getByRole('button',{name:'List s0 / decision: 175 records',exact:true}).press('Enter');
  await expect(page.locator('#scopeSel')).toHaveValue('s0');
  await expect(page.locator('#recordList tbody tr')).toHaveCount(175);
  expect(new Set(await page.locator('#recordList td[data-column="scope"]').allTextContents())).toEqual(new Set(['s0']));
  expect(new Set(await page.locator('#recordList td[data-column="type"]').allTextContents())).toEqual(new Set(['decision']));
});
