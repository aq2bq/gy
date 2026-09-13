import {test,expect} from '@playwright/test';
import {fixture,mouse} from '../helpers.mjs';

test('Overview links resolve the counted records and diagnostic severity',async({page},info)=>{
 await page.setViewportSize({width:1440,height:1000});
 await page.goto(fixture('reading'));
 for(const id of ['acSatisfied','acTotal','openQ','nextCount','lintErr','lintWarn']) {
  await page.goto(fixture('reading'));
  const count=Number(await page.locator('#'+id).textContent());
  await mouse(page,page.locator('#'+id).locator('..'));
  if(id.startsWith('lint')) {
   await expect(page.locator('#page-jams')).toBeVisible();
   await expect(page.locator('#jamLint .lint-sev')).toHaveCount(count);
   for(const severity of await page.locator('#jamLint .lint-sev').allTextContents())expect(severity).toBe(id==='lintErr'?'error':'warn');
  } else await expect(page.locator('#recordList tbody tr')).toHaveCount(count);
 }
 await page.goto(fixture('reading'));
 const states=await page.locator('#stateBars a').count();
 for(let i=0;i<states;i++){
  const link=page.locator('#stateBars a').nth(i),count=Number(await link.locator('em').textContent()),state=await link.getAttribute('title');
  await mouse(page,link);await expect(page.locator('#stateSel')).toHaveValue(state);await expect(page.locator('#recordList tbody tr')).toHaveCount(count);
  for(const value of await page.locator('#recordList td[data-column="status"]').allTextContents())expect(value).toBe(state);
 }
 await mouse(page,'#clearFilter');await mouse(page,'[data-kind="decision"]');
 await expect(page.locator('[data-kind="decision"]')).toHaveAttribute('aria-pressed','true');
 await expect(page.locator('[data-kind="all"]')).toHaveAttribute('aria-pressed','false');
 expect(await page.locator('[data-kind="decision"]').evaluate(e=>getComputedStyle(e).fontWeight)).toBe('700');
 await page.screenshot({path:info.outputPath('n27-1440.png')});
 await mouse(page,'#genealogy');await expect(page.locator('#genealogy')).toHaveText('Lineage: on');
 await mouse(page,'#genealogy');await expect(page.locator('#genealogy')).toHaveText('Lineage: off');
});

test('next source restores from the opened HTML rather than saved IDs',async({page})=>{
 await page.goto(fixture('reading'));await mouse(page,page.locator('#nextCount').locator('..'));
 await expect.poll(()=>decodeURIComponent(page.url())).toContain('"overview":"next"');
 const expected=await page.evaluate(()=>window.GY_DATA.next.map(n=>n.id).sort());
 await page.reload();expect((await page.locator('#recordList tbody a').allTextContents()).sort()).toEqual(expected);
 await mouse(page,'[data-kind="all"]');await expect(page.locator('#recordList tbody tr')).toHaveCount(24);
 await page.goBack();expect((await page.locator('#recordList tbody a').allTextContents()).sort()).toEqual(expected);
 await page.addInitScript(()=>{
  Object.defineProperty(window,'GY_DATA',{configurable:true,set(value){value.next=[];Object.defineProperty(window,'GY_DATA',{value,configurable:true});}});
 });
 await page.reload();await expect(page.locator('#recordList tbody tr')).toHaveCount(0);
 await expect(page.locator('#nextCount')).toHaveText('0');
});

test('pointer elements have native actions or registered role handlers',async({page})=>{
 await page.addInitScript(()=>{
  window.actionTargets=new WeakSet();const add=EventTarget.prototype.addEventListener;
  EventTarget.prototype.addEventListener=function(type,...args){if(type==='click' && this instanceof Element)window.actionTargets.add(this);return add.call(this,type,...args);};
 });
 for(const name of ['reading','large']){
  await page.goto(fixture(name));
  const failures=await page.evaluate(()=>[...document.querySelectorAll('*')].filter(el=>el.getClientRects().length&&getComputedStyle(el).cursor==='pointer').filter(el=>{
   const action=el.closest('a[href],button,[role="button"],[role="link"]');
   return !action || (!action.matches('a[href]')&&!window.actionTargets.has(action)&&!action.matches('button[data-sort],button[data-go]'));
  }).map(e=>e.outerHTML.slice(0,160)));
  expect(failures).toEqual([]);
  if(name==='large'){
   const count=await page.evaluate(()=>window.GY_DATA.lint.length);
   await mouse(page,page.locator('#lintWarn').locator('..'));
   await expect(page.locator('#page-jams')).toBeVisible();
   await mouse(page,page.getByRole('button',{name:'List s0 / decision: 175 records',exact:true}));
   await expect(page.locator('#jamLint .lint-sev')).toHaveCount(count);
  }
 }
});


test('all six row cells hit the native record link and open its target',async({page})=>{
 await page.setViewportSize({width:1440,height:1000});await page.goto(fixture('reading'));
 const row=page.locator('#recordList tbody tr').first();
 const id=await row.getAttribute('data-record-id');
 await expect(row).toHaveRole('row');
 expect(await row.evaluate(e=>getComputedStyle(e).cursor)).toBe('pointer');
 for(const column of ['id','type','title','scope','status','created']){
  const cell=row.locator('[data-column="'+column+'"]');
  const hit=await cell.evaluate(el=>{
   const r=el.getBoundingClientRect(),target=document.elementFromPoint(r.x+r.width/2,r.y+r.height/2);
   return {native:target instanceof HTMLAnchorElement,href:target.getAttribute('href'),id:target.getAttribute('data-record'),cursor:getComputedStyle(target).cursor};
  });
  expect(hit).toMatchObject({native:true,id,cursor:'pointer'});expect(hit.href).toContain('#view=');
  await mouse(page,cell);await expect(page.locator('.detail-id')).toHaveText(id);
  await mouse(page,'#closeDetail');await expect(page.locator('.detail-id')).not.toBeVisible();
 }
 await expect(page.locator('#recordList tbody tr')).toHaveCount(24);
 await expect(page.locator('#recordList tbody td')).toHaveCount(144);
});
