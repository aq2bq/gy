import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';
import { violations } from './architecture.mjs';

const roles = JSON.parse(readFileSync(new URL('../../crates/gy-serve/tests/asset-roles.json', import.meta.url)));
for (const [name, role] of Object.entries(roles)) {
  if (role === 'asset') continue;
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
