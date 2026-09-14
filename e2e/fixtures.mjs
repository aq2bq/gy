import {mkdirSync, writeFileSync, readFileSync, existsSync, rmSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {resolve, join} from 'node:path';
import {execFileSync} from 'node:child_process';
const here = fileURLToPath(new URL('.', import.meta.url));
const repo = resolve(here, '..');
execFileSync('cargo', ['build', '--locked', '-p', 'gy'], {cwd: repo, stdio: 'inherit'});
const bin = join(repo, 'target/debug', process.platform === 'win32' ? 'gy.exe' : 'gy');
const generated = join(here, '.generated');
const stampPath = join(generated, 'stamp.json');
const hash = path => createHash('sha256').update(readFileSync(path)).digest('hex');
const inputs = {binary: hash(bin), source: hash(fileURLToPath(import.meta.url))};
let previous;
try { previous = JSON.parse(readFileSync(stampPath, 'utf8')); } catch {}
if (!process.argv.includes('--force') && previous?.binary === inputs.binary && previous?.source === inputs.source
    && Array.isArray(previous.outputs) && previous.outputs.length > 0
    && previous.outputs.every(path => existsSync(join(generated, path)))
    && existsSync(join(generated, 'measurements.json'))) {
  console.log('Fixtures reused: binary and source hashes match.');
  process.exit(0);
}
mkdirSync(generated, {recursive:true});
rmSync(stampPath, {force:true});
const measurements = {};
const outputs = [];
function generate(name, nodes, edges) {
  const ledger = join(here, '.generated', name);
  rmSync(ledger, {recursive: true, force: true});
  mkdirSync(ledger, {recursive: true});
  const scopes = [...new Set(nodes.map(n => n.scope))];
  writeFileSync(join(ledger, 'gy.toml'), scopes.map(s => `[scopes.${s}]\nparent_issue = 1\n`).join('\n'));
  for (const n of nodes) {
    const kind = n.type || 'decision';
    const folder = {decision:'decisions',need:'needs',requirement:'requirements',question:'questions',criterion:'criteria',gate:'gates'}[kind];
    const dir = join(ledger, n.scope, folder);
    mkdirSync(dir, {recursive: true});
    const forward = edges.filter(e => e[0] === n.id).map(e => e[1]);
    const reverse = edges.filter(e => e[1] === n.id).map(e => e[0]);
    const attrs = {id:n.id, type:kind, created:'2026-09-13', scope:n.scope, title:`Synthetic ${n.id}`,
      ...(kind === 'decision' ? {decision_scope:'Automated public fixture only', supersedes:forward, 'superseded-by':reverse} : {}), ...n.attrs};
    writeFileSync(join(dir, `${n.id}.md`), '---\n' + Object.entries(attrs).map(([key,value]) => `${key}: ${JSON.stringify(value)}`).join('\n') + '\n---\n' + (n.body || 'Public synthetic fixture.\n'));

  }
  const start = performance.now();
  execFileSync(bin, ['-C', ledger, 'render', '--format', 'html', '--json']);
  const html = readFileSync(join(ledger, 'gy.html'), 'utf8');
  const payload = JSON.parse(html.split('window.GY_DATA = ')[1].split(';\n</script>')[0]);
  if (payload.nodes.length !== nodes.length || payload.edges.length !== edges.length) throw Error(`Empty or incomplete fixture: ${name}`);
  outputs.push(name + '/gy.html');
  measurements[name] = {nodes: nodes.length, edges: edges.length, bytes: Buffer.byteLength(html), generationMs: performance.now() - start};
}
// Disjoint 15-node chains: overview exceeds the threshold, neighborhoods do not.
const nodes = Array.from({length: 1000}, (_, i) => ({id: `D-${i + 1}`, scope: `s${Math.floor(i / 15) % 6}`}));
const edges = nodes.slice(1).flatMap((n, i) => (i + 1) % 15 ? [[nodes[i].id, n.id]] : []);
generate('large', nodes, edges);
generate('large-star', nodes, nodes.slice(1).map(n => ['D-1', n.id]));
const larger = Array.from({length:3000}, (_,i) => ({id:`D-${i+1}`,scope:`s${Math.floor(i/15)%6}`}));
generate('larger-star', larger, larger.slice(1).map(n => ['D-1', n.id]));
// 61 immediate neighbors; D-61 has higher degree than its peers. Three second-hop nodes.
const high = [{id: 'D-100', scope: 'star'}, ...Array.from({length: 64}, (_, i) => ({id: `D-${i + 1}`, scope: 'star'}))];
const star = Array.from({length: 61}, (_, i) => ['D-100', `D-${i + 1}`]);
star.push(['D-61', 'D-62'], ['D-61', 'D-63'], ['D-61', 'D-64']);
generate('high-degree', high, star);
// A 20-node neighborhood exercises wrapped titles at the requested review size.
const nearLabels=Array.from({length:20},(_,i)=>({id:`D-${200+i}`,scope:'labels',attrs:{title:i%3===0?'短い判断の記録':'適用する条件と判断した根拠を記録し関係する要求から設計の成立範囲を確かめる'}}));
generate('near-labels',nearLabels,nearLabels.slice(1).map(n=>['D-200',n.id]));
// Reading fixture includes all kinds, closed vocabularies, and lossless attributes.
const ja = 'この決定は記録された条件に基づいて適用する。表示された内容と実際の運用を照合し、変更の理由を確認する。'.repeat(8);
const en = 'This decision applies to the recorded conditions. Readers can compare the declaration with the explanation and retain the evidence for the next review. '.repeat(8);
const detailNodes = [
  {id:'D-501',scope:'reading',attrs:{title:'成立範囲と説明を分けて読む'.repeat(6),decision_scope:ja, pr_url:'https://example.test/pr/42', next_evidence:'確認結果を記録する',
    supersedes:[{id:'D-502',mark:'Prior contract'}], custom_false:false, custom_zero:0, custom_null:null,
    custom_nested:{flag:false, count:0, value:null, text:'<img src=x onerror=alert(1)>', list:['終端','none']},
    custom_long:'値を省略せずに保存する'.repeat(150)+'終端確認', custom_url:'javascript:alert(1)'},body:ja+'\n\n## 補足\n短い段落。'},
  {id:'D-502',scope:'reading',attrs:{title:'Read the declaration and its explanation',decision_scope:en,'superseded-by':[{id:'D-501',mark:'This decision applies to the recorded conditions.'}]},body:en},
  {id:'D-503',scope:'reading',attrs:{'superseded-by':[{id:'D-504',mark:'A passage absent from this body'}]}},
  {id:'D-504',scope:'reading',body:"## Structured reading\n\n| Left | Center | Right |\n| :--- | :---: | ---: |\n| **bold** | *emphasis* | `a|b` |\n| escaped\\|pipe | [safe link](https://example.test/docs(topic)#section \"Link title\") | 12 |\n\n- parent\n  1. first child\n     - grandchild\n  2. second child\n- sibling\n\n3. third item\n4. fourth item\n\n```javascript\nconst raw = \"<tag>&\";\n\twindow.__markdownCode = true;\n```\n\nInline `**not bold**` and ***combined emphasis***.\n\n[record link](#D-501) and [unsafe link](javascript:alert(1)).\n\n<script>window.__markdownRan = true;</script><img src=x onerror=\"window.__markdownRan=true\">\n\nSeparators: < > & \u2028 \u2029 end.\n"},
  {id:'N-501',type:'need',scope:'reading'},
  {id:'G-501',type:'gate',scope:'reading'},
  ...['fact','decision','non-decision'].map((method,i)=>({id:`Q-${501+i}`,type:'question',scope:'reading',attrs:{status:'closed',closed_by:method,closure_note:'Recorded reason'}})),
  {id:'Q-504',type:'question',scope:'reading',attrs:{status:'open'}},
  ...[true,false].map((satisfied,i)=>({id:`AC-${501+i}`,type:'criterion',scope:'reading',attrs:{satisfied}})),
  ...['unfiled','defining','awaiting-design','awaiting-approval','awaiting-implementation','awaiting-audit','awaiting-pr','awaiting-merge','awaiting-production','awaiting-cleanup','complete','future-state'].map((status,i)=>({id:`#${501+i}`,type:'requirement',scope:'reading',attrs:{status:status === 'awaiting-approval' ? status+' (owner review)' : status}}))
];
generate('reading', detailNodes, [['D-501','D-502']]);
writeFileSync(join(here, '.generated/measurements.json'), JSON.stringify(measurements, null, 2) + '\n');
console.log(measurements);

writeFileSync(stampPath, JSON.stringify({...inputs, outputs}, null, 2) + '\n');
