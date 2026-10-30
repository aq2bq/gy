/* The root (d-03ca): the one place that reads, draws the regions in order,
   turns events into intents, and names a read that did not answer. The first
   frame is drawn before any API call, so a slow or refused answer leaves a
   frame with a reason instead of an empty page (r-2d02). */
(function () {
  let words = null;
  let state = window.GyState.initial({
    lang: pickLang(),
    route: window.GyState.parseRoute(location.hash),
  });

  /* ?lang= wins, then the stored choice, then what the server sent (ac-b9b1). */
  function pickLang() {
    const asked = new URLSearchParams(location.search).get('lang');
    if (asked === 'en' || asked === 'ja') return asked;
    return localStorage.getItem('gy-lang') || (document.documentElement.lang === 'ja' ? 'ja' : 'en');
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

  /* The reads live in reads.js; the root keeps the pending marks. */
  const read = url => window.GyReads.read(state, url);

  /* The answer each screen needs. The shell and the band are the frame; the
     screen's own answer is what may be missing. */
  const SCREEN = {
    overview: ['overview'],
    cross: ['cross'],
    node: ['page', 'path'],
    missing: [],
  };
  const wanted = () => ['shell', 'band', ...(SCREEN[state.route.name] || [])];

  /* One read per wanted thing that is missing; dataArrived fills the state.
     A failure keeps the frame and records why, and never starts a second
     read for the same thing. */
  async function ensure(kind) {
    const url = window.GyReads.path(kind, state);
    if (!url || state[kind] || state.pending[kind]) return;
    /* The place the answer is for: a reply for a point or a node the reader
       has already left is dropped, not painted (n-32a9). */
    const ticket = window.GyState.routeKey(state.route) + `|${state.at}|${state.detail.depth}`;
    state = window.GyState.apply(state, { type: 'readStarted', kind });
    paint();
    try {
      const body = await window.GyReads.filled(state, kind, await (await read(url)).json());
      if (ticket !== window.GyState.routeKey(state.route) + `|${state.at}|${state.detail.depth}`) return;
      /* The band reads the synced copy's state from the shell (d-6d90). */
      if (kind === 'shell') window.GySync = body.sync || null;
      state = window.GyState.apply(state, { type: 'dataArrived', kind, body });
    } catch (failure) {
      if (ticket !== window.GyState.routeKey(state.route) + `|${state.at}|${state.detail.depth}`) return;
      state = window.GyState.apply(state, { type: 'readFailed', kind, reason: reasonOf(failure) });
    }
    paint();
  }

  /* The server's status and words when it gave them, else the browser's. */
  function reasonOf(failure) {
    const text = failure && failure.message ? failure.message : '';
    return text || word('loadFailed');
  }

  /* Ask for everything the frame wants; each answer paints as it comes. */
  function refresh() {
    return Promise.allSettled(wanted().map(ensure));
  }

  /* The reader asked again after a failure. */
  function retry() {
    state = window.GyState.apply(state, { type: 'readAgain' });
    paint();
    refresh();
  }

  /* A write arrived: state.js says what that makes stale (n-ca6d). */
  async function moved() {
    state = window.GyState.apply(state, { type: 'ledgerMoved' });
    paint();
    await refresh();
  }

  /* Draw the frame in order, then hand #main to the route's screen. */
  function paint() {
    if (window.GyHeader) window.GyHeader.render(state, document.querySelector('.top'), ui);
    if (window.GyBand) window.GyBand.render(state, document.querySelector('.scrub'), ui);
    if (window.GyPalette) window.GyPalette.render(state, document.getElementById('pal'), ui);
    paintPage();
  }

  function paintPage() {
    const main = document.getElementById('main');
    main.className = '';
    const name = state.route.name;
    if (name === 'overview' && window.GyOverview) window.GyOverview.render(state, main, ui);
    else if (name === 'cross' && window.GyCross) window.GyCross.render(state, main, ui);
    else if (name === 'node' && window.GyNode) window.GyNode.render(state, main, ui);
    else if (name === 'missing') main.innerHTML = `<div class="frame"><p class="hd">${word('notYet')}</p></div>`;
  }

  /* The insides of an intent: which ones need a read before the redraw. */
  async function run(intent) {
    state = window.GyState.apply(state, intent);
    if (intent.type === 'setLang') {
      localStorage.setItem('gy-lang', intent.value);
      paint();
      return;
    }
    if (intent.type === 'go') { paint(); await refresh(); return; }
    if (intent.type === 'detailDepth') { paint(); await refresh(); return; }
    if (intent.type === 'setAt') { preview(); scheduleRefresh(); return; }
    if (intent.type === 'copied') { scheduleCopy(); paint(); return; }
    if (intent.type === 'paletteOpen') { paint(); focusPalette(); paintPalette(); return; }
    if (intent.type === 'paletteQuery') { searchChanged(); return; }
    if (intent.type === 'paletteMove' || intent.type === 'paletteClose') { paintPalette(); return; }
    if (intent.type === 'liveChanged') { paintLive(); return; }
    paint();
  }

  /* The copy sign stays for a second, then the root clears it (n-6afd). */
  let copyTimer = null;
  function scheduleCopy() {
    if (copyTimer) clearTimeout(copyTimer);
    copyTimer = setTimeout(() => {
      copyTimer = null;
      state = window.GyState.apply(state, { type: 'copied', value: null });
      paint();
    }, 1000);
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
    paintPalette();
    try {
      const body = await (await window.GyReads.read(state, `/api/search?q=${encodeURIComponent(query)}`)).json();
      if (state.palette.query.trim() !== query) return;
      state = window.GyState.apply(state, { type: 'dataArrived', kind: 'search', body });
    } catch {
      if (state.palette.query.trim() !== query) return;
      state = window.GyState.apply(state, { type: 'dataArrived', kind: 'search', body: { hits: [] } });
    }
    paintPalette();
  }

  /* The connection mark, drawn by the header from state.live. */
  function setLive(on) {
    if (state.live === on) return;
    state = window.GyState.apply(state, { type: 'liveChanged', value: on });
    paintLive();
  }
  function paintLive() {
    if (window.GyHeader) window.GyHeader.render(state, document.querySelector('.top'), ui);
  }

  /* While the head is dragged: the band and the header follow at once. */
  function preview() {
    if (window.GyBand) window.GyBand.render(state, document.querySelector('.scrub'), ui);
    if (window.GyHeader) window.GyHeader.render(state, document.querySelector('.top'), ui);
    paintPage();
  }
  let refreshTimer = null;
  function scheduleRefresh() {
    if (refreshTimer) return;
    refreshTimer = setTimeout(() => { refreshTimer = null; refresh(); }, 60);
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
    if (window.GyState.routeKey(route) === window.GyState.routeKey(state.route)) return;
    run({ type: 'go', value: route });
  }

  function composedEnd() { composedAt = performance.now(); }
  function resized() {
    if (window.GyBand) window.GyBand.render(state, document.querySelector('.scrub'), ui);
  }

  /* What the regions may use: the read-only helpers and where a link goes. */
  const ledgerName = () => {
    const badge = document.querySelector('[data-testid="ledger"]');
    return badge ? badge.textContent.trim() : '';
  };
  const ui = {
    t: word, plural, scopeTag, when, day, tickAt,
    hue: name => window.GyState.hue(name, state.shell ? state.shell.scopes : []),
    canCopy: !!navigator.clipboard,
    navigate: hash => { location.hash = hash; },
    retry,
  };
  Object.defineProperty(ui, 'ledger', { get: ledgerName });

  /* The long wait for the next write lives in watch.js (n-39f2). */
  async function start() {
    /* The words are a local asset, not an API: read them first so the frame is
       written in the reader's language, then draw it before any answer. */
    try { words = await window.GyReads.words(); } catch { words = null; }
    paint();
    window.GyEvents.wire({ run, setAt, go, composing, composedRecently, composedEnd, resized, retry, state: () => state });
    refresh();
    window.GyWatch.follow(async () => { await moved(); }, on => setLive(on));
  }

  start();
})();
