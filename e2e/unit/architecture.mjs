// Inspect references and assignments, never comments, strings, or spelling.
import { parse } from 'acorn';
import { full } from 'acorn-walk';
import { analyze } from 'eslint-scope';

export function violations(source, name, roles) {
  const ast = parse(source, { ecmaVersion: 2022, ranges: true });
  const scopes = analyze(ast, { ecmaVersion: 2022, optimistic: true });
  const refs = new Map();
  for (const scope of scopes.scopes) {
    for (const ref of scope.references) refs.set(ref.identifier, ref);
  }
  const rootState = scopes.scopes.flatMap(scope => scope.variables)
    .filter(variable => variable.name === 'state' && variable.defs[0]?.type === 'Variable')
    .sort((a, b) => a.defs[0].name.start - b.defs[0].name.start)[0];
  const global = node => node?.type === 'Identifier' && !refs.get(node)?.resolved;
  const property = node => node.computed ? node.property.value : node.property.name;
  function path(node) {
    if (node?.type === 'Identifier') return global(node) ? node.name : null;
    if (node?.type !== 'MemberExpression') return null;
    const base = path(node.object);
    return base && property(node) ? `${base}.${property(node)}` : null;
  }
  function root(node) {
    while (node?.type === 'MemberExpression') node = node.object;
    return node;
  }
  const role = roles[name];
  const passive = role === 'region' || role === 'part';
  const errors = new Set();
  // Asset modules use Gy plus their capitalized file stem.
  const forbidden = new Set(Object.entries(roles)
    .filter(([file, kind]) => file !== name && ['region', 'core', 'network'].includes(kind))
    .map(([file]) => `Gy${file.replace('.js', '').replace(/^./, c => c.toUpperCase())}`));
  function mutation(target, value) {
    const id = root(target);
    if (id?.type !== 'Identifier' || id.name !== 'state') return;
    const ref = refs.get(id);
    const parameter = ref?.resolved?.defs.some(def => def.type === 'Parameter');
    if (passive && (parameter || !ref?.resolved)) errors.add('writes shared state');
    if (name === 'root.js' && ref?.resolved === rootState) {
      const factory = value?.type === 'CallExpression' && path(value.callee);
      if (target.type !== 'Identifier' || !['window.GyState.initial', 'window.GyState.apply'].includes(factory)) {
        errors.add('builds state outside GyState');
      }
    }
  }
  full(ast, node => {
    if (node.type === 'Identifier' && refs.has(node) && global(node)) {
      if (name === 'state.js' && ['document', 'fetch'].includes(node.name)) errors.add('state uses DOM or network');
      if (node.name === 'fetch' && role !== 'network') errors.add('network outside network role');
      if (role === 'region' && forbidden.has(node.name)) errors.add('region reaches another region or core');
    }
    if (node.type === 'MemberExpression') {
      const access = path(node);
      if (role === 'region' && access?.startsWith('window.') && forbidden.has(property(node))) errors.add('region reaches another region or core');
      if (passive && (access === 'document.getElementById' || access === 'window.document.getElementById' || property(node) === 'addEventListener')) errors.add('passive view owns DOM events');
      if (access === 'window.fetch' && role !== 'network') errors.add('network outside network role');
      if (name === 'state.js' && ['window.document', 'window.fetch'].includes(access)) errors.add('state uses DOM or network');
      if (passive && ['GyState.apply', 'window.GyState.apply'].includes(access)) errors.add('applies state outside root');
    }
    if (node.type === 'AssignmentExpression') mutation(node.left, node.right);
    if (node.type === 'UpdateExpression' || (node.type === 'UnaryExpression' && node.operator === 'delete')) mutation(node.argument);
    if (name === 'root.js' && node.type === 'VariableDeclarator' && node.id.name === 'state') mutation(node.id, node.init);
  });
  return [...errors];
}
