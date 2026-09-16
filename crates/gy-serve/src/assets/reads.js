/* The one place the app reads (d-03ca 第 3 段): the paths depend on the state it
   is handed, and every path gets `at` (n-32a9). It holds no state; the root
   keeps that and applies what comes back. */
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

  /* The page's answer, by route: a list, one node, or the history (n-5ca6). */
  function pagePath(state) {
    const route = state.route;
    if (route.name === 'list') return `/api/list?kind=${encodeURIComponent(route.arg)}${scopeParam(state) ? `&${scopeParam(state)}` : ''}`;
    if (route.name === 'node') return `/api/node/${encodeURIComponent(route.arg)}`;
    if (route.name === 'history') return historyPath(state);
    return null;
  }

  /* Every path this state wants; a kind with no path here is null. */
  function paths(state) {
    const only = scopeParam(state) ? `?${scopeParam(state)}` : '';
    const scope = scopeParam(state);
    return {
      shell: `/api/shell${only}`,
      now: `/api/now${only}`,
      map: `/api/graph${only}`,
      page: pagePath(state),
      band: '/api/ticks',
      rail: `/api/history?limit=10${scope ? `&${scope}` : ''}`,
    };
  }

  const path = (kind, state) => paths(state)[kind] || null;

  /* The one read: `at` is attached here for every path (n-32a9). */
  function read(state, url) {
    if (state.at === null) return fetch(url);
    const join = url.includes('?') ? '&' : '?';
    return fetch(`${url}${join}at=${state.at}`);
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
    if (kind === 'rail') return { rows: body.rows, labels: await labels(state, body.rows.map(row => row.node)) };
    if (kind === 'page' && state.route.name === 'history') return { ...body, labels: await labels(state, body.rows.map(entry => entry.node)) };
    return body;
  }

  window.GyReads = { path, read, filled };
})();