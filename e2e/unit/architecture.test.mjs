import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'bun:test';
import { violations } from './architecture.mjs';

const roles = JSON.parse(readFileSync(new URL('../../crates/gy-serve/tests/asset-roles.json', import.meta.url)));
for (const [name, role] of Object.entries(roles)) {
  if (!name.endsWith('.js')) continue;
  test(`${name} obeys its ${role} boundary`, () => {
    const source = readFileSync(new URL(`../../crates/gy-serve/src/assets/${name}`, import.meta.url), 'utf8');
    assert.deepEqual(violations(source, name, roles), []);
  });
}

test('local state, strings, comments, comparisons and indexed reads are allowed', () => {
  const source = `
    // document.getElementById('x'); fetch('x'); state.x = 1;
    const text = "window.GyReads.read(); state = x; fetch()";
    function label(closed) { const state = closed ? 'closed' : 'open'; return state; }
    function draw(state) { return state.x === 1 ? state['x'] : state.x; }
  `;
  assert.deepEqual(violations(source, 'node.js', roles), []);
  assert.deepEqual(violations('// document fetch\nconst text = "document fetch";', 'state.js', roles), []);
});

test('actual state mutations are rejected regardless of whitespace or property syntax', () => {
  for (const write of ['state.x=1', "state['x'] += 1", 'state.x++', 'delete state.x', 'state = {}']) {
    assert.ok(violations(`function draw(state) { ${write}; }`, 'node.js', roles).includes('writes shared state'), write);
  }
});

test('real dependency, DOM and network references are rejected', () => {
  for (const source of ["window['GyReads'].read()", "document['getElementById']('x')", "el.addEventListener('x', f)", "fetch ('x')", "window['fetch']('x')", 'window.GyState.apply(s, i)']) {
    assert.ok(violations(source, 'node.js', roles).length, source);
  }
  assert.ok(violations('const d = document;', 'state.js', roles).length);
});

test('only state factories construct root state', () => {
  assert.deepEqual(violations('let state=window.GyState.initial({}); state=window.GyState.apply(state, {});', 'root.js', roles), []);
  assert.deepEqual(violations('let state=window.GyState.initial({}); function label() { const state="open"; return state; }', 'root.js', roles), []);
  assert.ok(violations('let state={};', 'root.js', roles).length);
  assert.ok(violations('let state=window.GyState.initial({}); state.x=1;', 'root.js', roles).length);
});


test('a local alias of supplied state cannot be mutated', () => {
  assert.ok(violations('function draw(props) { const state = props.state; state.x = 1; }', 'node.js', roles).includes('writes shared state'));
});

test('an earlier local variable cannot hide root state', () => {
  const source = `(function () {
    function label() { const state = 'open'; return state; }
    let state = window.GyState.initial({}); state.x = 1;
  })();`;
  assert.ok(violations(source, 'root.js', roles).includes('builds state outside GyState'));
});

test('JavaScript cannot bypass checks through the asset role', () => {
  assert.ok(violations('fetch("/")', 'node.js', { ...roles, 'node.js': 'asset' }).includes('JavaScript must have a code role'));
});

test('a local fetch is distinct from a global network reference', () => {
  assert.deepEqual(violations('const fetch = () => {}; fetch("/");', 'node.js', roles), []);
  assert.deepEqual(violations('function f(fetch) { return fetch("/"); }', 'state.js', roles), []);
  assert.deepEqual(violations('fetch("/");', 'node.js', roles), ['network outside network role']);
});

test('a document parameter does not turn a local access into DOM ownership', () => {
  assert.deepEqual(violations('function f(document) { return document.getElementById("x"); }', 'node.js', roles), []);
  assert.deepEqual(violations('function f(document) { return document.x; }', 'state.js', roles), []);
  assert.deepEqual(violations('document.getElementById("x");', 'node.js', roles), ['passive view owns DOM events']);
});

test('a block-local window does not hide the global window outside the block', () => {
  assert.deepEqual(violations('{ const window = {}; window.fetch("/"); window.GyReads.read(); }', 'node.js', roles), []);
  assert.deepEqual(violations('{ let window = {}; window.document; }', 'state.js', roles), []);
  assert.deepEqual(violations('{ let window = {}; window.fetch("/"); } window.fetch("/");', 'node.js', roles), ['network outside network role']);
});

test('both lexical and var root declarations must use state factories', () => {
  for (const kind of ['let', 'const', 'var']) {
    assert.deepEqual(violations(`(function () { ${kind} state = {}; })();`, 'root.js', roles), ['builds state outside GyState']);
    assert.deepEqual(violations(`(function () { ${kind} state = window.GyState.initial({}); })();`, 'root.js', roles), []);
  }
});

test('root state and a state outside the immediate function stay distinct', () => {
  const source = 'let state = {}; (function () { let state = window.GyState.initial({}); function label() { const state = "open"; return state; } })();';
  assert.deepEqual(violations(source, 'root.js', roles), []);
});

test('compound violations retain their postorder', () => {
  assert.deepEqual(violations('function draw(state) { state.x = fetch("/"); }', 'node.js', roles), ['network outside network role', 'writes shared state']);
});

test('invalid syntax fails rather than passing with no violations', () => {
  assert.throws(() => violations('const = ;', 'node.js', roles), SyntaxError);
});
