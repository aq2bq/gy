/* The screen's one state and the intents that move it (d-03ca). No DOM and no
   fetch here: apply returns a new state and does nothing else. The three
   screens read their own answer (overview / cross / path), and the pages that
   came before them read theirs (now / list / eye / history / node), so a page
   that is still loading keeps the frame the root already drew (r-2d02). */
(function () {
  /* The route a hash names: overview (the first screen, d-e239), now (the
     home), cross, list, eye, history, node, or a route this build does not
     have. `query` carries the cross screen's narrowing (q, from, to, scope). */
  function parseRoute(hash) {
    const text = String(hash || '');
    const cut = text.indexOf('?');
    const path = cut < 0 ? text : text.slice(0, cut);
    const query = parseQuery(cut < 0 ? '' : text.slice(cut + 1));
    const list = path.match(/^#\/list\/(\w+)$/);
    const node = path.match(/^#\/n\/(.+)$/);
    const eye = path.match(/^#\/eye\/(wait|next|resume)$/);
    if (path === '' || path === '#/' || path === '#/overview') return { name: 'overview', arg: null, query };
    if (path === '#/now') return { name: 'now', arg: null, query };
    if (path === '#/cross') return { name: 'cross', arg: null, query };
    if (list) return { name: 'list', arg: list[1], query };
    if (eye) return { name: 'eye', arg: eye[1], query };
    if (node) return { name: 'node', arg: decodeURIComponent(node[1]), query };
    if (path === '#/history') return { name: 'history', arg: null, query };
    return { name: 'missing', arg: null, query };
  }

  /* `a=1&scope=b&scope=c` as `{a: '1', scope: ['b', 'c']}`. A repeated key is
     a list; `scope` is the one the cross API takes more than once. */
  function parseQuery(search) {
    const found = {};
    for (const pair of String(search || '').split('&')) {
      if (!pair) continue;
      const at = pair.indexOf('=');
      const key = decodeURIComponent(at < 0 ? pair : pair.slice(0, at));
      const value = decodeURIComponent((at < 0 ? '' : pair.slice(at + 1)).replace(/\+/g, ' '));
      if (key === 'scope') {
        if (!Array.isArray(found.scope)) found.scope = [];
        found.scope.push(value);
      } else {
        found[key] = value;
      }
    }
    return found;
  }

  /* What makes two routes the same place: the path and the whole query. */
  function routeKey(route) {
    const query = Object.keys(route.query || {})
      .sort()
      .map(key => {
        const value = route.query[key];
        return Array.isArray(value) ? `${key}=${value.join(',')}` : `${key}=${value}`;
      })
      .join('&');
    return `${route.name}|${route.arg || ''}|${query}`;
  }

  /* The first state: what the person chose, then the empty places the reads
     and the pages fill. Nothing here is judged from the record. */
  function initial(env) {
    return {
      lang: env.lang,
      /* The sidebar's scope: it narrows the shell, the home, a list, the
         history, and the rail, as it did before the three screens. */
      scope: env.scope,
      at: null,
      route: env.route,
      /* The rail's breakpoint, from app.css. */
      wide: env.wide,
      /* The nodes this tab has opened, for the detail tab and the way back. */
      trail: env.route.name === 'node' && env.route.arg ? [env.route.arg] : [],
      /* A list's open/all switch, and the history's filters. */
      list: { filter: 'open' },
      history: { actor: null, since: null },
      /* The thread's width: two stages each way, or three. */
      detail: { depth: 2 },
      palette: { open: false, query: '', selected: 0 },
      shell: null,
      band: null,
      page: null,
      now: null,
      rail: null,
      overview: null,
      cross: null,
      path: null,
      search: null,
      live: true,
      pending: {},
      failed: {},
      copied: null,
    };
  }

  /* A read that arrived: its place is filled, its marks cleared. */
  function arrived(state, kind, body) {
    const pending = { ...state.pending };
    const failed = { ...state.failed };
    delete pending[kind];
    delete failed[kind];
    return { ...state, [kind]: body, pending, failed };
  }

  /* The intents the root turns events into. A read's data never moves here. */
  function apply(state, intent) {
    switch (intent.type) {
      case 'setLang':
        return { ...state, lang: intent.value };
      case 'setScope':
        return { ...state, scope: intent.value, shell: null, now: null, page: null, rail: null, pending: {}, failed: {} };
      case 'setAt':
        return {
          ...state,
          at: intent.value,
          shell: null,
          now: null,
          page: null,
          rail: null,
          overview: null,
          cross: null,
          path: null,
          pending: {},
          failed: {},
        };
      case 'go': {
        const route = intent.value;
        const last = state.trail[state.trail.length - 1];
        const trail = route.name === 'node' && route.arg && route.arg !== last
          ? [...state.trail, route.arg].slice(-20)
          : state.trail;
        return {
          ...state,
          route,
          trail,
          page: null,
          now: null,
          rail: null,
          overview: null,
          cross: null,
          path: null,
          pending: {},
          failed: {},
          /* A list opens on its open rows, whatever the last one showed. */
          list: route.name === 'list' ? { ...state.list, filter: 'open' } : state.list,
          palette: { ...state.palette, open: false, query: '', selected: 0 },
          search: null,
        };
      }
      case 'setWide':
        return { ...state, wide: intent.value };
      case 'listFilter':
        return { ...state, list: { ...state.list, filter: intent.value } };
      case 'historyActor':
        return { ...state, history: { ...state.history, actor: intent.value }, page: null, pending: {}, failed: {} };
      case 'historySince':
        return { ...state, history: { ...state.history, since: intent.value }, page: null, pending: {}, failed: {} };
      case 'detailDepth':
        return { ...state, detail: { depth: intent.value }, path: null, pending: {}, failed: {} };
      /* A write moved the ledger. The band and the rail are of the old head; at
         the head the answers are too. */
      case 'ledgerMoved': {
        const head = state.at === null;
        return {
          ...state,
          band: null,
          rail: null,
          ...(head ? { shell: null, now: null, page: null, overview: null, cross: null, path: null, pending: {}, failed: {} } : {}),
        };
      }
      case 'liveChanged':
        return { ...state, live: intent.value };
      /* The palette's hits belong to the word that asked for them. */
      case 'paletteOpen':
        return { ...state, palette: { ...state.palette, open: true, query: '', selected: 0 }, search: null };
      case 'paletteClose':
        return { ...state, palette: { ...state.palette, open: false }, search: null };
      case 'paletteQuery':
        return { ...state, palette: { ...state.palette, query: intent.value, selected: 0 }, search: null };
      case 'paletteMove':
        return { ...state, palette: { ...state.palette, selected: intent.value } };
      case 'copied':
        return { ...state, copied: intent.value };
      /* A read that is out: the mark keeps it from being asked twice. */
      case 'readStarted':
        return { ...state, pending: { ...state.pending, [intent.kind]: true } };
      /* A read that did not answer: the frame stays and says why (r-2d02). */
      case 'readFailed': {
        const pending = { ...state.pending };
        delete pending[intent.kind];
        return { ...state, pending, failed: { ...state.failed, [intent.kind]: intent.reason } };
      }
      /* The reader asked again: the marks go and the root asks again. */
      case 'readAgain':
        return { ...state, pending: {}, failed: {} };
      case 'dataArrived':
        return arrived(state, intent.kind, intent.body);
      default:
        return state;
    }
  }

  /* Pure text and time helpers (no DOM, no fetch), so the root and the regions
     share one copy. */
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

  /* The scope's hue among this ledger's scopes: a hue already taken is skipped,
     in the server's scope order, so two scopes are told apart by colour. */
  function hue(name, scopes) {
    const text = String(name ?? '');
    const names = (scopes || []).map(item => item.name);
    const wanted = names.indexOf(text);
    let found = SCOPE_HUES[hashIndex(text)];
    if (wanted >= 0) {
      const taken = new Set();
      for (let at = 0; at <= wanted; at++) {
        let slot = hashIndex(names[at]);
        while (taken.has(slot) && taken.size < SCOPE_HUES.length) slot = (slot + 1) % SCOPE_HUES.length;
        if (at === wanted) found = SCOPE_HUES[slot];
        else taken.add(slot);
      }
    }
    return found;
  }

  /* The scope badge (n-4e5a). */
  function scopeTag(name, scopes) {
    return `<span class="sb" data-testid="scopeBadge" style="--sb:${hue(name, scopes)}">${esc(String(name ?? ''))}</span>`;
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

  window.GyState = { initial, apply, parseRoute, routeKey, esc, plural, scopeTag, hue, when, day, tickAt };
})();
