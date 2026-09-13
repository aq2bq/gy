import {test,expect} from '@playwright/test';
import {fixture} from '../helpers.mjs';
async function metrics(page) {
 return page.evaluate(()=>{
  const viewport=document.querySelector('#svg').getBoundingClientRect();
  const labels=[...document.querySelectorAll('#svg [data-node-id] .nlabel')],boxes=labels.map(e=>e.getBoundingClientRect());
  let intersections=0;
  for(let i=0;i<boxes.length;i++)for(let j=i+1;j<boxes.length;j++){const a=boxes[i],b=boxes[j];if(a.left<b.right&&b.left<a.right&&a.top<b.bottom&&b.top<a.bottom)intersections++;}
  const truncated=labels.filter(e=>e.dataset.truncated==='true');
  return {labels:labels.length,full:labels.length-truncated.length,truncated:truncated.length,minRetained:truncated.length?Math.min(...truncated.map(e=>Number(e.dataset.retained))):null,minCharacterHeight:Math.min(...labels.flatMap(e=>[...e.children].map(t=>t.getBoundingClientRect().height))),intersections,outside:boxes.filter(r=>r.left<viewport.left||r.right>viewport.right||r.top<viewport.top||r.bottom>viewport.bottom).length};
 });
}
for(const [name,count,state] of [['near-labels',20,{focus:[{id:'D-200',radius:1}],selected:'D-200'}],['reading',24,{}],['high-degree',60,{focus:[{id:'D-100',radius:1}],selected:'D-100'}]])test('labels wrap, remain readable and all fit: '+name,async({page},info)=>{
 await page.setViewportSize({width:1440,height:1000});
 await page.goto(fixture(name)+'#view='+encodeURIComponent(JSON.stringify(state)));
 const m=await metrics(page);await info.attach('label-measurements',{body:JSON.stringify(m),contentType:'application/json'});
 expect(m.labels).toBe(count);expect(m.intersections).toBe(0);expect(m.outside).toBe(0);expect(m.minCharacterHeight).toBeGreaterThanOrEqual(11);
 if(m.truncated)expect(m.minRetained).toBeGreaterThanOrEqual(20);
 for(const lines of await page.locator('#svg [data-node-id] .nlabel').evaluateAll(es=>es.map(e=>e.children.length)))expect(lines).toBeLessThanOrEqual(4);
 await page.screenshot({path:info.outputPath('n24-'+name+'-1440.png')});
});
test('cluster labels fit at readable character height',async({page})=>{
 await page.setViewportSize({width:1440,height:1000});await page.goto(fixture('large'));
 const m=await page.locator('#svg text.nlabel').evaluateAll(es=>{const v=document.querySelector('#svg').getBoundingClientRect();return es.map(e=>{const b=e.getBoundingClientRect();return {height:b.height,inside:b.left>=v.left&&b.right<=v.right&&b.top>=v.top&&b.bottom<=v.bottom};});});
 expect(m.length).toBeGreaterThan(0);for(const x of m){expect(x.height).toBeGreaterThanOrEqual(11);expect(x.inside).toBe(true);}
});
test('individual edge labels avoid node titles and unavailable placements use details',async({page},info)=>{
 await page.setViewportSize({width:1440,height:1000});
 await page.goto(fixture('near-labels')+'#view='+encodeURIComponent(JSON.stringify({focus:[{id:'D-200',radius:1}],selected:'D-200'})));
 const measure=()=>page.evaluate(()=>{
  const nodes=[...document.querySelectorAll('#svg .node .nlabel')].map(e=>e.getBoundingClientRect());
  const labels=[...document.querySelectorAll('#svg .elabel')].map(e=>e.getBoundingClientRect());
  return {nodes:nodes.length,labels:labels.length,edges:document.querySelectorAll('#svg line[data-lbl]').length,intersections:labels.reduce((n,a)=>n+nodes.filter(b=>a.left<b.right&&b.left<a.right&&a.top<b.bottom&&b.top<a.bottom).length,0)};
 });
 const normal=await measure();
 expect(normal.nodes).toBe(20);expect(normal.labels).toBeGreaterThan(0);expect(normal.edges).toBe(19);expect(normal.intersections).toBe(0);
 await info.attach('edge-label-measurements',{body:JSON.stringify(normal),contentType:'application/json'});
 await page.screenshot({path:info.outputPath('n29-edge-labels-1440.png')});
 const relationships=await page.locator('.detail-relations').allTextContents();
 expect(relationships.length).toBeGreaterThan(0);
 // An oversized measured label cannot fit near these edges. Keep the edges and
 // the exact detail relationships while suppressing their colliding labels.
 await page.addStyleTag({content:'svg .elabel{font-size:1000px}'});
 await page.setViewportSize({width:1441,height:1000});
 await expect(page.locator('#svg .elabel')).toHaveCount(0);
 expect((await measure()).edges).toBe(normal.edges);
 expect(await page.locator('.detail-relations').allTextContents()).toEqual(relationships);
});
