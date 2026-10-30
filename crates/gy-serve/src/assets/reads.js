/* The one place the app reads (d-03ca): the paths depend on the state it is
   handed, and every path gets `at` (n-32a9). It holds no state; the root keeps
   that and applies what comes back. A read that does not answer throws with
   the status and the server's reason, so the screen can say why (r-2d02). */
(function () {
  const scopeParam = state => {
    const scope = state.scope;
    return scope && scope !== 'all' ? `scope=${encodeURIComponent(scope)}` : '';
  };

  /* The history page's query, from the state's filters (n-4a08). */
  function historyPath(state) {
    const parts = [];
    const scope = scopeParam(state);
    if (scope) parts.push(scope);
    if (state.history.actor) parts.push(`actor=${encodeURIComponent(state.history.actor)}`);
    if (state.history.since !== null) parts.push(`since=${state.history.since}`);
    return `/api/history${parts.length ? `?${parts.join('&')}` : ''}`;
  }

  /* The cross screen's narrowing, exactly as the hash carries it: q, from, to,
     and a repeated scope. An empty screen asks for nothing and gets the grid. */
  function crossQuery(route) {
    const query = route.query || {};
    const parts = [];
    if (query.q) parts.push(`q=${encodeURIComponent(query.q)}`);
    if (query.from) parts.push(`from=${encodeURIComponent(query.from)}`);
    if (query.to) parts.push(`to=${encodeURIComponent(query.to)}`);
    for (const scope of Array.isArray(query.scope) ? query.scope : query.scope ? [query.scope] : []) {
      if (scope) parts.push(`scope=${encodeURIComponent(scope)}`);
    }
    return parts.length ? `?${parts.join('&')}` : '';
  }

  /* The page's answer, by route: a list, one node, the history. */
  function pagePath(state) {
    const route = state.route;
    if (route.name === 'list') {
      const scope = scopeParam(state);
      return `/api/list?kind=${encodeURIComponent(route.arg)}${scope ? `&${scope}` : ''}`;
    }
    if (route.name === 'node') return `/api/node/${encodeURIComponent(route.arg)}`;
    if (route.name === 'history') return historyPath(state);
    return null;
  }

  /* Every path this state wants; a kind with no path here is null. The three
     screens keep their own answers: the overview never waits for the cross. */
  function paths(state) {
    const route = state.route;
    const scope = scopeParam(state);
    const only = scope ? `?${scope}` : '';
    return {
      shell: `/api/shell${only}`,
      /* The sidebar's three eyes count on every page, as they did (n-c122). */
      now: `/api/now${only}`,
      band: '/api/ticks',
      rail: `/api/history?limit=10${scope ? `&${scope}` : ''}`,
      page: pagePath(state),
      overview: route.name === 'overview' ? '/api/overview' : null,
      cross: route.name === 'cross' ? `/api/cross${crossQuery(route)}` : null,
      path: route.name === 'node'
        ? `/api/node/${encodeURIComponent(route.arg)}/path?depth=${state.detail.depth}`
        : null,
    };
  }

  const path = (kind, state) => paths(state)[kind] || null;

  /* The server's own words for a refusal, when it sent any (503 carries one).
     A JSON body names a reason; a plain one is the reason itself. */
  async function why(res) {
    const text = await res.text();
    try {
      const body = JSON.parse(text);
      return body.reason || body.error || body.message || res.statusText;
    } catch {
      return text.trim() || res.statusText;
    }
  }

  /* The one read: `at` is attached here for every path (n-32a9), and a refusal
     becomes an error the root can name. */
  async function read(state, url) {
    const full = state.at === null ? url : `${url}${url.includes('?') ? '&' : '?'}at=${state.at}`;
    const res = await fetch(full);
    if (!res.ok) {
      const error = new Error(`${res.status} ${await why(res)}`);
      error.status = res.status;
      throw error;
    }
    return res;
  }

  /* The app's own words (n-39f2): a read like any other, so it lives here. */
  async function words() {
    const res = await fetch('/assets/i18n.json');
    return res.json();
  }

  /* The aliases of some nodes, for the rows that show names (n-b963, n-4a08). */
  async function labels(state, ids) {
    const unique = [...new Set(ids.filter(Boolean))];
    const found = {};
    for (let at = 0; at < unique.length; at += 200) {
      const chunk = unique.slice(at, at + 200);
      const body = await (await read(state, `/api/labels?ids=${chunk.join(',')}`)).json();
      Object.assign(found, body.labels || {});
    }
    return found;
  }

  /* What a body needs beyond the answer: the band's left end, and the labels
     the rail's or the history page's rows show. */
  async function filled(state, kind, body) {
    if (kind === 'band') return { ...body, min: body.ticks.length ? body.ticks[0].seq : 1 };
    if (kind === 'rail') return { ...body, labels: await labels(state, body.rows.map(row => row.node)) };
    if (kind === 'page' && state.route.name === 'history') {
      return { ...body, labels: await labels(state, body.rows.map(entry => entry.node)) };
    }
    return body;
  }

  window.GyReads = { path, read, filled, words };
})();
