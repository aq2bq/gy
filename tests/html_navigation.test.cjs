// Run with: node --test tests/html_navigation.test.cjs
// State/selection regression tests; browser hit testing is verified separately.
const {test} = require('node:test');
const assert = require('node:assert/strict');
const {readFileSync} = require('node:fs');
const {join} = require('node:path');
const vm = require('node:vm');

function fixture() {
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
  for (const file of ['state.js', 'selection.js', 'navigation.js']) {
    vm.runInContext(readFileSync(join(__dirname, '../crates/gy-core/src/html', file), 'utf8'), context);
  }
  return {run: code => JSON.parse(JSON.stringify(vm.runInContext(code, context))), counters};
}

test('descent, duplicate focus, one-step and multi-step return preserve stored radii', () => {
  const f = fixture();
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

test('filters, search and lineage remain independent when the focus is hidden and when returning', () => {
  const f = fixture();
  f.run("startHop('D-0'); startHop('Q-1'); genealogyMode = true; searchText = 'D-0'; scopeSel.value = 'a'; visibleNodes()");
  assert.deepEqual(f.run('visibleNodes()'), ['D-0']);
  assert.equal(f.run('currentFocus().id'), 'Q-1');
  f.run('returnFocus(1); focusHistory');
  assert.deepEqual(f.run('[genealogyMode, searchText, scopeSel.value]'), [true,'D-0','a']);
  f.run('typeState.decision = false; visibleNodes()');
  assert.deepEqual(f.run('visibleNodes()'), []);
  assert.equal(f.run('currentFocus().id'), 'D-0');
  f.run('returnFocus(0); focusHistory');
  assert.deepEqual(f.run('[genealogyMode, searchText, scopeSel.value, typeState.decision]'), [true,'D-0','a',false]);
});

test('invalid navigation does not alter history, detail or redraw count', () => {
  const f = fixture();
  f.run("startHop('missing'); returnFocus(-1); returnFocus(1); returnFocus(0.5); focusHistory");
  assert.deepEqual(f.run('focusHistory'), [{id: null, radius: null}]);
  assert.equal(f.counters.draw, 0);
  assert.equal(f.counters.detail, null);
});

test('adaptive radius is selected on entry and retained on return', () => {
  const f = fixture();
  f.run("for (let i=1;i<=30;i++) {adj['D-0']['D-'+i]='supersedes'; adj['D-'+i]['D-0']='superseded-by'; adj['D-'+i]['D-'+(i+30)]='supersedes'; adj['D-'+(i+30)]['D-'+i]='superseded-by';} startHop('D-0'); focusHistory");
  const first = f.run('currentFocus()');
  assert.equal(first.radius, 1, 'two hops exceed the 60-node limit in this fixture');
  assert.ok(f.run("reachable('D-0', currentFocus().radius).size") <= 60);
  f.run("startHop('D-70'); adj['D-0'] = {}; returnFocus(1); focusHistory");
  assert.equal(f.run("pickHop('D-0').n"), 5, 'recomputing would now produce a different radius');
  assert.deepEqual(f.run('currentFocus()'), first);
  assert.equal(f.counters.draw, 3, 'one draw per navigation operation');
});

function viewportFixture(positions) {
  const ids = Object.keys(positions);
  const context = vm.createContext({
    NODES: ids.map(id => ({id})), EDGES: [],
    svg: {addEventListener() {}, getBoundingClientRect() {return {width: 800, height: 600};}},
    window: {addEventListener() {}}, root: {setAttribute() {}},
    visibleNodes() {return ids;}, ensureForce() {}, lodFromScale() {return 'near';}, draw() {}
  });
  for (const file of ['state.js','viewport.js']) {
    vm.runInContext(readFileSync(join(__dirname, '../crates/gy-core/src/html', file), 'utf8'), context);
  }
  vm.runInContext('forceLayout = ' + JSON.stringify(positions), context);
  return code => JSON.parse(JSON.stringify(vm.runInContext(code, context)));
}

test('isolated and two-node automatic fits stay finite and capped; manual zoom may go further', () => {
  for (const positions of [{a: {x: 0, y: 0}}, {a: {x: 0, y: 0}, b: {x: 20, y: 0}}]) {
    const run = viewportFixture(positions);
    const fitted = run('fitTransform(); ({scale, translate, viewSource})');
    assert.equal(fitted.scale, 2);
    assert.ok(Number.isFinite(fitted.translate.x) && Number.isFinite(fitted.translate.y));
    assert.equal(fitted.viewSource, 'fit');
    assert.deepEqual(run("zoomBy(1.6); [scale, viewSource]"), [3.2, 'manual']);
  }
});

test('an empty selection leaves a finite transform', () => {
  const run = viewportFixture({});
  assert.deepEqual(run('fitTransform(); [scale, translate.x, translate.y, viewSource]'), [1,0,0,'fit']);
});


test('isolated-focus notice is based on adjacency, not the number of visible nodes', () => {
  const f = fixture();
  f.run("startHop('D-10'); renderNavigation(['D-10']); currentFocus()");
  assert.doesNotMatch(f.run("document.getElementById('focusStatus').textContent"), /No connections/);
  f.run("adj['D-10'] = {}; renderNavigation(['D-10']); currentFocus()");
  assert.match(f.run("document.getElementById('focusStatus').textContent"), /No connections in this graph/);
  f.run('renderNavigation([]); currentFocus()');
  assert.match(f.run("document.getElementById('focusStatus').textContent"), /Focus hidden/);
  assert.equal(f.run('currentFocus().id'), 'D-10');
});


test('a focused destination stays centered when its neighborhood cannot fit at readable scale', () => {
  const run = viewportFixture({a: {x: 0, y: -4000}, b: {x: 200, y: 1000}});
  assert.deepEqual(run("focusHistory.push({id:'b',radius:1}); fitTransform(); [scale, (800/2-translate.x)/scale, (600/2-translate.y)/scale]"), [1,200,1000]);
});


test('without a displayed focus, readable fit centers the selected bounds', () => {
  const run = viewportFixture({a: {x: 0, y: -4000}, b: {x: 200, y: 1000}});
  assert.deepEqual(run("fitTransform(); [scale, (800/2-translate.x)/scale, (600/2-translate.y)/scale]"), [1,100,-1500]);
  assert.deepEqual(run("focusHistory.push({id:'hidden',radius:1}); fitTransform(); [scale, (800/2-translate.x)/scale, (600/2-translate.y)/scale]"), [1,100,-1500]);
});
