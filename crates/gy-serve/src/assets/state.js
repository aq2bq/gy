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
      /* The nodes this tab has opened, for the "came from" mark (n-...). The
         first page counts, so the second one can name it. */
      trail: env.route.name === 'node' && env.route.arg ? [env.route.arg] : [],
      list: { filter: 'open' },
      history: { actor: null, since: null },
      graph: { cam: { x: 0, y: 0, k: 1 }, target: null, selected: null, hover: null },
      /* The node page's connections map: its own camera, so the graph page's
         and the node's never move each other (n-9ca9). */
      ego: { cam: { x: 0, y: 0, k: 1 } },
      palette: { open: false, query: '', selected: 0 },
      shell: null,
      now: null,
      map: null,
      labels: null,
      band: null,
      page: null,
      rail: null,
      search: null,
      live: true,
      pending: {},
      copied: null,
    };
  }

  /* A read that arrived: its place is filled and its pending mark cleared. */
  function arrived(state, kind, body) {
    const pending = { ...state.pending };
    delete pending[kind];
    if (kind === 'now') {
      /* The band and the write rows read the shared copy's sync from here
         (n-94bb); a local ledger leaves it null. */
      window.GySync = (body.resume && body.resume.sync) || null;
    }
    return { ...state, [kind]: body, pending };
  }

  /* The intents the root turns events into. A read's data never moves here. */
  function apply(state, intent) {
    switch (intent.type) {
      case 'setLang':
        return { ...state, lang: intent.value };
      case 'setScope':
        return { ...state, scope: intent.value, shell: null, now: null, map: null, labels: null, page: null, rail: null, graph: { ...state.graph, target: null, selected: null, hover: null } };
      case 'setAt':
        return { ...state, at: intent.value, shell: null, now: null, map: null, labels: null, page: null, rail: null, graph: { ...state.graph, target: null, selected: null, hover: null } };
      case 'go': {
        const route = intent.value;
        const last = state.trail[state.trail.length - 1];
        const trail = route.name === 'node' && route.arg && route.arg !== last
          ? [...state.trail, route.arg].slice(-20)
          : state.trail;
        /* Another node opens at the whole figure; the same node keeps its
           camera through a redraw (n-9ca9). */
        const ego = route.name === 'node' && route.arg !== state.route.arg
          ? { cam: { x: 0, y: 0, k: 1 } }
          : state.ego;
        const list = route.name === 'list' ? { ...state.list, filter: 'open' } : state.list;
        return { ...state, route, trail, ego, page: null, list };
      }
      case 'setWide':
        return { ...state, wide: intent.value };
      case 'listFilter':
        return { ...state, list: { ...state.list, filter: intent.value } };
      case 'historyActor':
        return { ...state, history: { ...state.history, actor: intent.value }, page: null };
      case 'historySince':
        return { ...state, history: { ...state.history, since: intent.value }, page: null };
      case 'graphCam':
        return { ...state, graph: { ...state.graph, cam: intent.value } };
      case 'graphTarget':
        return { ...state, graph: { ...state.graph, target: intent.value } };
      case 'mapCam':
        return { ...state, ego: { ...state.ego, cam: intent.value } };
      case 'graphSelect':
        return { ...state, graph: { ...state.graph, selected: intent.value }, page: null };
      case 'graphHover':
        return { ...state, graph: { ...state.graph, hover: intent.value } };
      /* The palette's hits belong to the word that asked for them: opening, a
         new word, and closing all leave the old answer behind (n-ca6d). */
      case 'paletteOpen':
        return { ...state, palette: { ...state.palette, open: true, query: '', selected: 0 }, search: null };
      case 'paletteClose':
        return { ...state, palette: { ...state.palette, open: false }, search: null };
      case 'paletteQuery':
        return { ...state, palette: { ...state.palette, query: intent.value, selected: 0 }, search: null };
      case 'paletteMove':
        return { ...state, palette: { ...state.palette, selected: intent.value } };
      /* A write moves the ledger. The band and the rail are of the old head, and
         at the head the shell, now, and page too. The graph page keeps its map
         and labels: its camera and selection hold their picture (n-88b2). */
      case 'ledgerMoved': {
        const head = state.at === null;
        const graph = state.route.name === 'graph';
        return {
          ...state,
          band: null,
          rail: null,
          ...(head ? { shell: null, now: null, page: null, map: graph ? state.map : null, labels: graph ? state.labels : null } : {}),
        };
      }
      case 'liveChanged':
        return { ...state, live: intent.value };
      /* The name just copied, for the mark's sign; the root's timer clears it
         (n-6afd). */
      case 'copied':
        return { ...state, copied: intent.value };
      /* A read that is out: the mark keeps it from being asked twice, and
         `dataArrived` clears it when the answer comes (n-ca6d). */
      case 'readStarted':
        return { ...state, pending: { ...state.pending, [intent.kind]: true } };
      case 'dataArrived':
        return arrived(state, intent.kind, intent.body);
      default:
        return state;
    }
  }

  /* Pure text and time helpers (no DOM, no fetch), so root and the regions share
   one copy. They live here with the state because they judge nothing else. */
  const PLURAL = { Need: 'Needs', Question: 'Questions', Decision: 'Decisions', Requirement: 'Requirements', Criterion: 'Criteria' };
  const SCOPE_HUES = [10, 40, 95, 140, 175, 210, 262, 320];
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));

  function plural(kind, translate) {
    return translate(PLURAL[kind] || kind);
  }

  /* A name's own hue, before anything is taken into account (n-d29f). */
  function hashIndex(name) {
    let hash = 0x811c9dc5;
    for (let index = 0; index < name.length; index++) {
      hash = Math.imul(hash ^ name.charCodeAt(index), 0x01000193);
    }
    return (hash >>> 0) % SCOPE_HUES.length;
  }

  /* The scope badge (n-4e5a): a hue already taken in this ledger is skipped, in
     the server's scope order, so two scopes are told apart by colour. */
  function scopeTag(name, scopes) {
    const text = String(name ?? '');
    const names = (scopes || []).map(item => item.name);
    const wanted = names.indexOf(text);
    let hue = SCOPE_HUES[hashIndex(text)];
    if (wanted >= 0) {
      const taken = new Set();
      for (let at = 0; at <= wanted; at++) {
        let slot = hashIndex(names[at]);
        while (taken.has(slot) && taken.size < SCOPE_HUES.length) slot = (slot + 1) % SCOPE_HUES.length;
        if (at === wanted) hue = SCOPE_HUES[slot];
        else taken.add(slot);
      }
    }
    return `<span class="sb" style="--sb:${hue}">${esc(text)}</span>`;
  }

  const loc = lang => (lang === 'ja' ? 'ja-JP' : 'en-GB');
  const when = (at, lang) => new Date(at * 1000).toLocaleString(loc(lang), { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
  const day = (at, lang) => new Date(at * 1000).toLocaleDateString(loc(lang));

  /* The tick at or before a sequence, so a point between writes has a time. */
  function tickAt(ticks, seq) {
    let found = null;
    for (const tick of ticks || []) {
      if (tick.seq > seq) break;
      found = tick;
    }
    return found;
  }

  window.GyState = { initial, apply, parseRoute, esc, plural, scopeTag, when, day, tickAt };
})();