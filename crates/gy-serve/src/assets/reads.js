/* The one place the app reads (d-03ca): the paths depend on the state it is
   handed, and every path gets `at` (n-32a9). It holds no state; the root keeps
   that and applies what comes back. A read that does not answer throws with
   the status and the server's reason, so the screen can say why (r-2d02). */
(function () {
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

  /* Every path this state wants; a kind with no path here is null. The three
     screens keep their own answers: the overview never waits for the cross. */
  function paths(state) {
    const route = state.route;
    return {
      shell: '/api/shell',
      band: '/api/ticks',
      overview: route.name === 'overview' ? '/api/overview' : null,
      cross: route.name === 'cross' ? `/api/cross${crossQuery(route)}` : null,
      page: route.name === 'node' ? `/api/node/${encodeURIComponent(route.arg)}` : null,
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

  /* What a body needs beyond the answer: the band's left end. */
  async function filled(state, kind, body) {
    if (kind === 'band') return { ...body, min: body.ticks.length ? body.ticks[0].seq : 1 };
    return body;
  }

  window.GyReads = { path, read, filled, words };
})();
