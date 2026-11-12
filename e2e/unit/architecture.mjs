// Inspect references and assignments, never comments, strings, or spelling.
import { parse, walk } from 'yuku';

export function violations(source, name, roles) {
  const file = parse(source, { sourceType: 'script', preserveParens: false, analyze: true });
  const syntax = file.diagnostics.filter(diagnostic => diagnostic.severity === 'error');
  if (syntax.length) throw new SyntaxError(syntax.map(diagnostic => diagnostic.message).join('\n'));
  const ast = file.program;
  const wrapper = ast.body.find(node => node.type === 'ExpressionStatement'
    && node.expression.type === 'CallExpression'
    && node.expression.callee.type === 'FunctionExpression')?.expression.callee;
  // Lexical declarations live in functionBody; var lives in function. find
  // checks only that scope, so an outer or nested state cannot replace it.
  const rootState = wrapper
    ? file.scopeOf(wrapper.body)?.find('state') ?? file.scopeOf(wrapper)?.find('state')
    : file.rootScope.find('state');
  const global = node => node?.type === 'Identifier' && !file.referenceOf(node)?.binding;
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
  if (name.endsWith('.js') && role === 'asset') errors.add('JavaScript must have a code role');
  // Asset modules use Gy plus their capitalized file stem.
  const forbidden = new Set(Object.entries(roles)
    .filter(([file, kind]) => file !== name && ['region', 'core', 'network'].includes(kind))
    .map(([file]) => `Gy${file.replace('.js', '').replace(/^./, c => c.toUpperCase())}`));
  function mutation(target, value, declared) {
    const id = root(target);
    if (id?.type !== 'Identifier' || id.name !== 'state') return;
    const binding = declared ?? file.referenceOf(id)?.binding;
    if (passive) errors.add('writes shared state');
    if (name === 'root.js' && rootState && binding === rootState) {
      const factory = value?.type === 'CallExpression' && path(value.callee);
      if (target.type !== 'Identifier' || !['window.GyState.initial', 'window.GyState.apply'].includes(factory)) {
        errors.add('builds state outside GyState');
      }
    }
  }
  // Leave preserves the previous postorder, including the order of errors.
  walk(ast, { leave(node) {
    if (node.type === 'Identifier' && file.referenceOf(node) && global(node)) {
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
    if (name === 'root.js' && node.type === 'VariableDeclarator' && node.id.name === 'state') mutation(node.id, node.init, file.bindingOf(node.id));
  } });
  return [...errors];
}
