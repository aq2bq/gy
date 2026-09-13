// Run with: node --experimental-vm-modules --test tests/html_navigation.test.cjs
// State/selection regression tests; browser hit testing is verified separately.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const {execFileSync} = require('node:child_process');
const {join, resolve} = require('node:path');
const vm = require('node:vm');

const sourceRoot = resolve(__dirname, '../crates/gy-core/ui/src');
const sources = JSON.parse(execFileSync('bun', [join(sourceRoot, '../tests/transpile.ts')], {encoding:'utf8'}));
async function loadModules(context, entries, stubs) {
  const modules = new Map();
  function moduleFor(file) {
    if (!modules.has(file)) modules.set(file, new vm.SourceTextModule(stubs[file] ?? sources[file], {context, identifier:file}));
    return modules.get(file);
  }
  const entry = new vm.SourceTextModule(entries.map(file => `export * from './${file}';`).join('\n'), {context, identifier:join(sourceRoot,'test-entry.ts')});
  await entry.link((name, ref) => moduleFor(resolve(ref.identifier, '..', name) + '.ts'));
  await entry.evaluate();
  for (const name of Object.getOwnPropertyNames(entry.namespace)) Object.defineProperty(context, name, {get:()=>entry.namespace[name]});
}
async function fixture() {
  const nodes = Array.from({length: 80}, (_, i) => ({id: 'D-' + i, type: 'decision', title: 'Decision ' + i, scope: 'a', attrs: {}}));
  nodes.push({id: 'Q-1', type: 'question', scope: 'b', attrs: {}, status: 'open'});
  const edges = Array.from({length: 79}, (_, i) => ({source: 'D-' + i, target: 'D-' + (i+1), label: 'supersedes', reverse: 'superseded-by'}));
  edges.push({source: 'Q-1', target: 'D-0', label: 'closes', reverse: 'closed-by'});
  const element = () => ({
    value: '', textContent: '', dataset: {}, style: {}, addEventListener() {}, setAttribute() {},
    replaceChildren() {}, appendChild(child) {this.lastElementChild = child;}, scrollIntoView() {}
  });
  const elements = new Map();
  const counters = {draw: 0, detail: null};
  const context = vm.createContext({
    NODES: nodes, EDGES: edges, byId: Object.fromEntries(nodes.map(n => [n.id, n])),
    document: {createElement: element, createTextNode: text => ({textContent: text}), getElementById(id) {
      if (!elements.has(id)) elements.set(id, element());
      return elements.get(id);
    }},
    typeState: {decision: true, question: true}, scopeSel: {value: ''}, stateSel: {value: ''},
    computeMatches: q => new Set(nodes.filter(n => n.id === q).map(n => n.id)),
    closeClusterPanel() {}, showDetail(id) {counters.detail = id;}, hideDetail() {counters.detail = null;},
    redraw() {counters.draw++;}
  });
  context.window = {GY_DATA:{nodes, edges, dependencies:[]}};
  await loadModules(context, ['state','graph/selection','navigation'], {
    [join(sourceRoot,'components.ts')]: 'export const redraw = globalThis.redraw;',
    [join(sourceRoot,'filters.ts')]: 'export const computeMatches = globalThis.computeMatches; export const scopeSel = globalThis.scopeSel; export const stateSel = globalThis.stateSel;',
    [join(sourceRoot,'detail/index.ts')]: 'export const showDetail = globalThis.showDetail; export const hideDetail = globalThis.hideDetail;',
    [join(sourceRoot,'graph/clusters.ts')]: 'export const closeClusterPanel = globalThis.closeClusterPanel;'
  });
  return {run: code => JSON.parse(JSON.stringify(vm.runInContext(code, context)) ?? 'null'), counters};
}

test('descent, duplicate focus, one-step and multi-step return preserve stored radii', async () => {
  const f = await fixture();
  assert.deepEqual(f.run('currentFocus()'), {id: null, radius: null});
  f.run("startHop('D-10'); startHop('D-15'); startHop('D-20'); focusHistory");
  const history = f.run('focusHistory');
  assert.equal(history.length, 4);
  assert.deepEqual(history.slice(1).map(e => e.radius), [5,5,5]);
  f.run("startHop('D-20'); focusHistory");
  assert.deepEqual(f.run('focusHistory'), history);
  f.run('returnFocus(2); focusHistory');
  assert.deepEqual(f.run('currentFocus()'), history[2]);
  f.run("startHop('D-25'); returnFocus(1); focusHistory");
  assert.deepEqual(f.run('focusHistory'), history.slice(0,2));
  assert.equal(f.counters.detail, 'D-10');
  f.run('returnFocus(0); focusHistory');
  assert.deepEqual(f.run('focusHistory'), [{id: null, radius: null}]);
  assert.equal(f.counters.detail, null);
});

test('filters, search and lineage remain independent when the focus is hidden and when returning', async () => {
  const f = await fixture();
  f.run("startHop('D-0'); startHop('Q-1'); setGenealogyMode(true); setSearchText('D-0'); setFilter('scopeSel','a'); scopeSel.value = 'a'; visibleNodes()");
  assert.deepEqual(f.run('visibleNodes()'), ['D-0']);
  assert.equal(f.run('currentFocus().id'), 'Q-1');
  f.run('returnFocus(1); focusHistory');
  assert.deepEqual(f.run('[genealogyMode, searchText, scopeSel.value]'), [true,'D-0','a']);
  f.run("setType('decision', false); visibleNodes()");
  assert.deepEqual(f.run('visibleNodes()'), []);
  assert.equal(f.run('currentFocus().id'), 'D-0');
  f.run('returnFocus(0); focusHistory');
  assert.deepEqual(f.run('[genealogyMode, searchText, scopeSel.value, typeState.decision]'), [true,'D-0','a',false]);
});

test('invalid navigation does not alter history, detail or redraw count', async () => {
  const f = await fixture();
  f.run("startHop('missing'); returnFocus(-1); returnFocus(1); returnFocus(0.5); focusHistory");
  assert.deepEqual(f.run('focusHistory'), [{id: null, radius: null}]);
  assert.equal(f.counters.draw, 0);
  assert.equal(f.counters.detail, null);
});

test('adaptive radius is selected on entry and retained on return', async () => {
  const f = await fixture();
  f.run("for (let i=1;i<=30;i++) {adj['D-0']['D-'+i]='supersedes'; adj['D-'+i]['D-0']='superseded-by'; adj['D-'+i]['D-'+(i+30)]='supersedes'; adj['D-'+(i+30)]['D-'+i]='superseded-by';} startHop('D-0'); focusHistory");
  const first = f.run('currentFocus()');
  assert.equal(first.radius, 1, 'two hops exceed the 60-node limit in this fixture');
  assert.ok(f.run("reachable('D-0', currentFocus().radius).size") <= 60);
  f.run("startHop('D-70'); adj['D-0'] = {}; returnFocus(1); focusHistory");
  assert.equal(f.run("pickHop('D-0').n"), 5, 'recomputing would now produce a different radius');
  assert.deepEqual(f.run('currentFocus()'), first);
  assert.equal(f.counters.draw, 3, 'one draw per navigation operation');
});

async function viewportFixture(positions) {
  const ids = Object.keys(positions);
  const context = vm.createContext({
    NODES: ids.map(id => ({id})), EDGES: [],
    svg: {addEventListener() {}, getBoundingClientRect() {return {width: 800, height: 600};}},
    window: {addEventListener() {}}, root: {setAttribute() {}},
    visibleNodes() {return ids;}, ensureForce() {}, lodFromScale() {return 'near';}, draw() {}
  });
  context.window.GY_DATA = {nodes:ids.map(id=>({id})), edges:[], dependencies:[]};
  await loadModules(context, ['state','graph/viewport'], {
    [join(sourceRoot,'graph/svg.ts')]: 'export const svg = globalThis.svg; export const root = globalThis.root;',
    [join(sourceRoot,'graph/layout.ts')]: 'export const ensureForce = globalThis.ensureForce;',
    [join(sourceRoot,'graph/selection.ts')]: 'export const visibleNodes = globalThis.visibleNodes; export const lodFromScale = globalThis.lodFromScale;',
    [join(sourceRoot,'graph/draw.ts')]: 'export const draw = globalThis.draw; export function overviewGrid() { throw new Error("not an overview fixture"); }'
  });
  vm.runInContext('setForceLayout(' + JSON.stringify(positions) + ')', context);
  return code => JSON.parse(JSON.stringify(vm.runInContext(code, context)) ?? 'null');
}

test('isolated and two-node automatic fits stay finite and capped; manual zoom may go further', async () => {
  for (const positions of [{a: {x: 0, y: 0}}, {a: {x: 0, y: 0}, b: {x: 20, y: 0}}]) {
    const run = await viewportFixture(positions);
    const fitted = run('fitTransform(); ({scale, translate, viewSource})');
    assert.equal(fitted.scale, 2);
    assert.ok(Number.isFinite(fitted.translate.x) && Number.isFinite(fitted.translate.y));
    assert.equal(fitted.viewSource, 'fit');
    assert.deepEqual(run("zoomBy(1.6); [scale, viewSource]"), [3.2, 'manual']);
  }
});

test('an empty selection leaves a finite transform', async () => {
  const run = await viewportFixture({});
  assert.deepEqual(run('fitTransform(); [scale, translate.x, translate.y, viewSource]'), [1,0,0,'fit']);
});


test('isolated-focus notice is based on adjacency, not the number of visible nodes', async () => {
  const f = await fixture();
  f.run("startHop('D-10'); renderNavigation(['D-10']); currentFocus()");
  assert.doesNotMatch(f.run("document.getElementById('focusNote').textContent"), /No connections/);
  f.run("adj['D-10'] = {}; renderNavigation(['D-10']); currentFocus()");
  assert.match(f.run("document.getElementById('focusNote').textContent"), /No connections in this graph/);
  f.run('renderNavigation([]); currentFocus()');
  assert.match(f.run("document.getElementById('focusNote').textContent"), /Focus hidden/);
  assert.equal(f.run('currentFocus().id'), 'D-10');
});


test('a focused destination stays centered when its neighborhood cannot fit at readable scale', async () => {
  const run = await viewportFixture({a: {x: 0, y: -4000}, b: {x: 200, y: 1000}});
  assert.deepEqual(run("enterFocus('b',1); fitTransform(); [scale, (800/2-translate.x)/scale, (600/2-translate.y)/scale]"), [1,200,1000]);
});


test('without a displayed focus, readable fit centers the selected bounds', async () => {
  const run = await viewportFixture({a: {x: 0, y: -4000}, b: {x: 200, y: 1000}});
  assert.deepEqual(run("fitTransform(); [scale, (800/2-translate.x)/scale, (600/2-translate.y)/scale]"), [1,100,-1500]);
  assert.deepEqual(run("enterFocus('hidden',1); fitTransform(); [scale, (800/2-translate.x)/scale, (600/2-translate.y)/scale]"), [1,100,-1500]);
});

test('focused selection caps at 60 using distance, degree and ID; filters still hide the focus', async () => {
  const f = await fixture();
  f.run(`for (const id of Object.keys(adj)) adj[id] = {};
    for (let i=1; i<80; i++) { adj['D-0']['D-'+i]='supersedes'; adj['D-'+i]['D-0']='superseded-by'; }
    adj['D-79']['D-78']='supersedes'; adj['D-78']['D-79']='superseded-by';
    for (const id of Object.keys(adj)) degree[id] = Object.keys(adj[id]).length;
    startHop('D-0'); currentFocus();`);
  const selection = f.run('focusedSelection()');
  assert.equal(selection.ids.length, 60);
  assert.equal(selection.omitted.length, 20);
  assert.deepEqual(selection.ids.slice(0, 3), ['D-0', 'D-78', 'D-79']);
  assert.deepEqual(selection.ids.slice(3), Array.from({length:77}, (_,i) => 'D-'+(i+1)).sort().slice(0,57));
  assert.equal(f.run('candidateNodes().length'), 80);
  f.run("setSearchText('D-1')");
  assert.deepEqual(f.run('visibleNodes()'), ['D-1']);
  assert.equal(f.run('currentFocus().id'), 'D-0');
  f.run("setSearchText(''); returnFocus(0); currentFocus()");
  assert.equal(f.run('visibleNodes().length'), 81);
});
