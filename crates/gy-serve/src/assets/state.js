/* The web UI's one state and the intents that move it (d-03ca, 第 2 段). No DOM
   and no fetch here: apply returns a new state and does nothing else. */
(function () {
  /* The route a hash names: now / list / eye / node / history / graph. `arg`
     carries the kind, the eye key, the node id, or the graph focus. */
  function parseRoute(hash) {
    const list = hash.match(/^#\/list\/(\w+)$/);
    const node = hash.match(/^#\/n\/(.+)$/);
    const graph = hash.match(/^#\/graph(?:\/(.+))?$/);
    const eye = hash.match(/^#\/eye\/(wait|next|resume)$/);
    if (hash === '' || hash === '#/') return { name: 'now', arg: null };
    if (list) return { name: 'list', arg: list[1] };
    if (graph) return { name: 'graph', arg: graph[1] ? decodeURIComponent(graph[1]) : null };
    if (eye) return { name: 'eye', arg: eye[1] };
    if (node) return { name: 'node', arg: decodeURIComponent(node[1]) };
    if (hash === '#/history') return { name: 'history', arg: null };
    return { name: 'missing', arg: null };
  }

  /* The first state: what the person chose, then the empty places the reads
     and the pages fill. */
  function initial(env) {
    return {
      lang: env.lang,
      scope: env.scope,
      at: null,
      route: env.route,
      wide: env.wide,
      list: { filter: 'open' },
      history: { actor: null, since: null },
      graph: { cam: { x: 0, y: 0, k: 1 }, selected: null },
      palette: { open: false, query: '', selected: 0 },
      shell: null,
      now: null,
      band: null,
      page: null,
      rail: null,
      pending: {},
    };
  }

  /* A read that arrived: its place is filled and its pending mark cleared. */
  function arrived(state, kind, body) {
    const pending = { ...state.pending };
    delete pending[kind];
    return { ...state, [kind]: body, pending };
  }

  /* The intents the root turns events into. A read's data never moves here. */
  function apply(state, intent) {
    switch (intent.type) {
      case 'setLang':
        return { ...state, lang: intent.value };
      case 'setScope':
        return { ...state, scope: intent.value, shell: null, now: null, page: null, rail: null };
      case 'setAt':
        return { ...state, at: intent.value, shell: null, now: null, page: null, rail: null };
      case 'go':
        return { ...state, route: intent.value, page: null };
      case 'setWide':
        return { ...state, wide: intent.value };
      case 'listFilter':
        return { ...state, list: { ...state.list, filter: intent.value }, page: null };
      case 'historyActor':
        return { ...state, history: { ...state.history, actor: intent.value }, page: null };
      case 'historySince':
        return { ...state, history: { ...state.history, since: intent.value }, page: null };
      case 'graphCam':
        return { ...state, graph: { ...state.graph, cam: intent.value } };
      case 'graphSelect':
        return { ...state, graph: { ...state.graph, selected: intent.value } };
      case 'paletteOpen':
        return { ...state, palette: { ...state.palette, open: true, query: '', selected: 0 } };
      case 'paletteClose':
        return { ...state, palette: { ...state.palette, open: false } };
      case 'paletteQuery':
        return { ...state, palette: { ...state.palette, query: intent.value, selected: 0 } };
      case 'paletteMove':
        return { ...state, palette: { ...state.palette, selected: intent.value } };
      case 'ledgerMoved':
        return { ...state, shell: null, now: null, page: null, rail: null };
      case 'dataArrived':
        return arrived(state, intent.kind, intent.body);
      default:
        return state;
    }
  }

  window.GyState = { initial, apply, parseRoute };
})();