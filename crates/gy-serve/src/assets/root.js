/* The root (d-03ca, 第 2 段): the one place that reads, draws the regions in
   order, and turns events into intents. Regions never reach back here; the
   `window.GyShell` at the bottom is a bridge for the pages, gone in 第 3 段. */
(function () {
  /* The rail's breakpoint; app.css holds the same number (change both). */
  const WIDE = window.matchMedia('(min-width: 1560px)');

  let words = null;
  let state = window.GyState.initial({
    lang: pickLang(),
    scope: pickScope(),
    route: window.GyState.parseRoute(location.hash),
    wide: WIDE.matches,
  });

  /* ?lang= wins, then the stored choice, then what the server sent (ac-b9b1). */
  function pickLang() {
    const asked = new URLSearchParams(location.search).get('lang');
    if (asked === 'en' || asked === 'ja') return asked;
    return localStorage.getItem('gy-lang') || (document.documentElement.lang === 'ja' ? 'ja' : 'en');
  }
  function pickScope() {
    return sessionStorage.getItem('gy-scope') || 'all';
  }

  /* A word by key; a dotted key walks nested objects (`st.Need.open`). */
  function word(key) {
    if (!words) return key;
    const found = key.split('.').reduce((value, part) => (value ? value[part] : undefined), words[state.lang]);
    return found ?? key;
  }

  /* The shared pure helpers, bound to this state (state.js holds them). */
  const plural = kind => window.GyState.plural(kind, word);
  const scopeTag = name => window.GyState.scopeTag(name, state.shell ? state.shell.scopes : []);
  const when = at => window.GyState.when(at, state.lang);
  const day = at => window.GyState.day(at, state.lang);
  const tickAt = seq => window.GyState.tickAt(state.band ? state.band.ticks : [], seq);

  /* An IME's own key, not the app's (n-87ab). */
  let composedAt = 0;
  const composing = event => event.isComposing || event.keyCode === 229;
  const composedRecently = () => performance.now() - composedAt < 50;

  /* The one read: `at` is attached here for every path (n-32a9). */
  function read(path) {
    if (state.at === null) return fetch(path);
    const join = path.includes('?') ? '&' : '?';
    return fetch(`${path}${join}at=${state.at}`);
  }

  const scopeParam = () => {
    const scope = state.scope;
    return scope && scope !== 'all' ? `scope=${encodeURIComponent(scope)}` : '';
  };
  const only = () => (scopeParam() ? `?${scopeParam()}` : '');
  const PATHS = {
    shell: () => `/api/shell${only()}`,
    now: () => `/api/now${only()}`,
    map: () => `/api/graph${only()}`,
    band: () => '/api/ticks',
    rail: () => `/api/history?limit=10${scopeParam() ? `&${scopeParam()}` : ''}`,
  };

  /* One read per wanted thing that is missing; dataArrived fills the state. */
  async function ensure(kind) {
    if (state[kind] || state.pending[kind] || !PATHS[kind]) return;
    state = { ...state, pending: { ...state.pending, [kind]: true } };
    try {
      const body = await filled(kind, await (await read(PATHS[kind]())).json());
      state = window.GyState.apply(state, { type: 'dataArrived', kind, body });
    } catch {
      state = window.GyState.apply(state, { type: 'dataArrived', kind, body: null });
    }
  }

  /* The band's left end, and the rail's labels, need a second look. */
  async function filled(kind, body) {
    if (kind === 'band') return { ...body, min: body.ticks.length ? body.ticks[0].seq : 1 };
    if (kind !== 'rail') return body;
    const ids = [...new Set(body.rows.map(row => row.node).filter(Boolean))];
    const labels = {};
    if (ids.length) Object.assign(labels, (await (await read(`/api/labels?ids=${ids.join(',')}`)).json()).labels);
    return { rows: body.rows, labels };
  }

  async function reload() {
    const wants = ['shell', 'now', 'band', 'rail'];
    if (state.route.name === 'now') wants.push('map');
    await Promise.all(wants.map(ensure));
    paint();
  }

  /* Live updates: the band always follows the log; the rest only at the head. */
  async function moved() {
    state = { ...state, band: null, rail: null };
    if (state.at === null) state = window.GyState.apply(state, { type: 'ledgerMoved' });
    await reload();
  }

  /* Draw the regions in order, then hand #main to the route's page. */
  function paint() {
    document.documentElement.lang = state.lang;
    if (window.GySidebar) window.GySidebar.render(state, document.querySelector('.side'), ui);
    if (window.GyTopbar) window.GyTopbar.render(state, document.getElementById('crumb'), ui);
    if (window.GyBand) window.GyBand.render(state, document.querySelector('.scrub'), ui);
    if (window.GyRail) window.GyRail.render(state, document.getElementById('railBody'), ui);
    if (window.GyPalette) window.GyPalette.render(state, document.getElementById('pal'), ui);
    paintPage();
  }

  function paintPage() {
    const main = document.getElementById('main');
    main.className = '';
    const name = state.route.name;
    const arg = state.route.arg;
    if (name === 'now' && window.GyNow) window.GyNow.render(state, main, ui);
    else if (name === 'list' && window.GyList) window.GyList.draw(arg);
    else if (name === 'eye' && window.GyEye) window.GyEye.draw(arg);
    else if (name === 'node' && window.GyNode) window.GyNode.draw(arg);
    else if (name === 'history' && window.GyHistory) window.GyHistory.draw();
    else if (name === 'graph' && window.GyGraph) window.GyGraph.draw(arg);
    else if (name === 'missing') main.textContent = word('notYet');
  }

  /* The insides of an intent: which ones need a read before the redraw. */
  let reloadTimer = null;
  async function run(intent) {
    state = window.GyState.apply(state, intent);
    if (intent.type === 'setScope') { await reload(); return; }
    if (intent.type === 'go') { await reload(); return; }
    if (intent.type === 'setAt') { preview(); scheduleReload(); return; }
    if (intent.type === 'paletteOpen') {
      state = { ...state, search: null };
      paintPalette();
      focusPalette();
      return;
    }
    if (intent.type === 'paletteQuery') { searchChanged(); return; }
    if (intent.type === 'paletteMove' || intent.type === 'paletteClose') { paintPalette(); return; }
    if (intent.type === 'liveChanged') { paintLive(); return; }
    paint();
  }

  function paintPalette() {
    if (window.GyPalette) window.GyPalette.render(state, document.getElementById('pal'), ui);
  }

  function focusPalette() {
    const query = document.getElementById('palq');
    if (query) query.focus();
  }

  /* Each keystroke asks once; a reply for an older query is dropped. */
  async function searchChanged() {
    const query = state.palette.query.trim();
    if (!query) {
      state = window.GyState.apply(state, { type: 'dataArrived', kind: 'search', body: { hits: [] } });
      paintPalette();
      return;
    }
    state = { ...state, search: null };
    paintPalette();
    const body = await (await read(`/api/search?q=${encodeURIComponent(query)}`)).json();
    if (state.palette.query.trim() !== query) return;
    state = window.GyState.apply(state, { type: 'dataArrived', kind: 'search', body });
    paintPalette();
  }

  /* The connection mark, drawn by the sidebar and the rail from state.live. */
  function setLive(on) {
    if (state.live === on) return;
    state = window.GyState.apply(state, { type: 'liveChanged', value: on });
    paintLive();
  }
  function paintLive() {
    if (window.GySidebar) window.GySidebar.render(state, document.querySelector('.side'), ui);
    if (window.GyRail) window.GyRail.render(state, document.getElementById('railBody'), ui);
  }

  /* While the head is dragged: the band and the clock follow at once. */
  function preview() {
    if (window.GyBand) window.GyBand.render(state, document.querySelector('.scrub'), ui);
    if (window.GySidebar) window.GySidebar.render(state, document.querySelector('.side'), ui);
  }
  function scheduleReload() {
    if (reloadTimer) return;
    reloadTimer = setTimeout(() => { reloadTimer = null; reload(); }, 60);
  }

  /* The band's head: a number of the log, or null for the head (n-32a9). */
  function setAt(seq) {
    const ticks = state.band;
    const max = ticks ? ticks.max : 0;
    const min = ticks ? ticks.min : 1;
    let value = seq === null ? null : Math.max(min, Math.min(max, Math.round(seq)));
    if (value !== null && value >= max) value = null;
    run({ type: 'setAt', value });
  }

  function go(route) {
    if (route.name === state.route.name && route.arg === state.route.arg) return;
    run({ type: 'go', value: route });
  }

  /* Events: the wiring is in events.js; here is what it may call. */
  function composedEnd() { composedAt = performance.now(); }
  function resized() {
    if (window.GyBand) window.GyBand.render(state, document.querySelector('.scrub'), ui);
  }

  /* What the regions may use; read-only helpers and the elements palette needs
     to place the search (it owns the frame, not its holders). */
  const ui = {
    t: word, plural, scopeTag, tickAt, when, day,
    search: document.getElementById('searchbtn'),
    railSearch: document.getElementById('railSearch'),
    topbar: document.querySelector('.topbar'),
  };

  /* The bridge for the pages until 第 3 段 makes them regions (n-6c63). */
  window.GyShell = {
    t: word,
    lang: () => state.lang,
    scope: () => state.scope,
    get at() { return state.at; },
    set at(value) { state = window.GyState.apply(state, { type: 'setAt', value }); },
    read,
    plural,
    scopeTag,
    composing,
    composedRecently,
    setEyes: () => {}, /* the three boxes come from state.now; nothing to push (n-45ca) */
  };

  /* The long wait for the next write (was live.js, n-478f): 30 s cut, 2 s
     retry, and state.live when the line drops. */
  const CUT = 30000;
  const RETRY = 2000;
  let known = null;
  const pause = ms => new Promise(done => setTimeout(done, ms));

  async function waitPast(after) {
    const control = new AbortController();
    const timer = setTimeout(() => control.abort(), CUT);
    try {
      const res = await fetch(`/api/wait?after=${after}`, { signal: control.signal });
      return (await res.json()).seq;
    } finally {
      clearTimeout(timer);
    }
  }

  async function liveLoop() {
    for (;;) {
      if (known === null) {
        try {
          known = (await (await fetch('/api/shell')).json()).seq;
        } catch {
          setLive(false);
          await pause(RETRY);
          continue;
        }
      }
      let seq;
      try {
        seq = await waitPast(known);
      } catch {
        setLive(false);
        await pause(RETRY);
        continue;
      }
      setLive(true);
      if (seq <= known) continue;
      known = seq;
      try {
        await moved();
      } catch {
        setLive(false);
      }
    }
  }

  async function start() {
    const res = await fetch('/assets/i18n.json');
    words = await res.json();
    window.GyEvents.wire({
      run, setAt, go, composing, composedRecently, composedEnd, resized,
      state: () => state,
    });
    WIDE.addEventListener('change', () => run({ type: 'setWide', value: WIDE.matches }));
    await reload();
    liveLoop();
  }

  window.GyRoot = { moved };
  start();
})();