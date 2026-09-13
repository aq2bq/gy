import {mkdirSync, writeFileSync, readFileSync} from 'node:fs';
import {fileURLToPath} from 'node:url';
import {resolve, join} from 'node:path';
import {execFileSync} from 'node:child_process';
const here = fileURLToPath(new URL('.', import.meta.url));
const repo = resolve(here, '..');
execFileSync('cargo', ['build', '--locked', '-p', 'gy'], {cwd: repo, stdio: 'inherit'});
const bin = join(repo, 'target/debug', process.platform === 'win32' ? 'gy.exe' : 'gy');
const measurements = {};
function generate(name, nodes, edges) {
  const ledger = join(here, '.generated', name);
  mkdirSync(ledger, {recursive: true});
  const scopes = [...new Set(nodes.map(n => n.scope))];
  writeFileSync(join(ledger, 'gy.toml'), scopes.map(s => `[scopes.${s}]\nparent_issue = 1\n`).join('\n'));
  for (const n of nodes) {
    const dir = join(ledger, n.scope, 'decisions');
    mkdirSync(dir, {recursive: true});
    const forward = edges.filter(e => e[0] === n.id).map(e => e[1]);
    const reverse = edges.filter(e => e[1] === n.id).map(e => e[0]);
    writeFileSync(join(dir, `${n.id}.md`), `---\nid: ${n.id}\ntype: decision\ncreated: 2026-09-13\nscope: ${n.scope}\ntitle: Synthetic ${n.id}\ndecision_scope: Automated public fixture only\nsupersedes: ${JSON.stringify(forward)}\nsuperseded-by: ${JSON.stringify(reverse)}\n---\nPublic synthetic fixture.\n`);
  }
  const start = performance.now();
  execFileSync(bin, ['-C', ledger, 'render', '--format', 'html', '--json']);
  const html = readFileSync(join(ledger, 'gy.html'), 'utf8');
  const payload = JSON.parse(html.split('window.GY_DATA = ')[1].split(';\n</script>')[0]);
  if (payload.nodes.length !== nodes.length || payload.edges.length !== edges.length) throw Error(`Empty or incomplete fixture: ${name}`);
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
writeFileSync(join(here, '.generated/measurements.json'), JSON.stringify(measurements, null, 2) + '\n');
console.log(measurements);
