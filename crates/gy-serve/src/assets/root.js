/* The root (d-03ca, 第 2 段): the one place that reads, draws the regions in
   order, and turns events into intents. Regions never reach back here; the
   `window.GyShell` at the bottom is a bridge for the pages, gone in 第 3 段. */
(function () {
  const PLURAL = { Need: 'Needs', Question: 'Questions', Decision: 'Decisions', Requirement: 'Requirements', Criterion: 'Criteria' };
  const SCOPE_HUES = [10, 40, 95, 140, 175, 210, 262, 320];
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
  const plural = kind => word(PLURAL[kind] || kind);

  /* A name's own hue, before anything is taken into account (n-d29f). */
  function hashIndex(name) {
    let hash = 0x811c9dc5;
    for (let index = 0; index < name.length; index++) {
      hash = Math.imul(hash ^ name.charCodeAt(index), 0x01000193);
    }
    return (hash >>> 0) % SCOPE_HUES.length;
  }

  /* The scope badge (n-4e5a): a hue already taken in this ledger is skipped,
     in the server's scope order, so two scopes are told apart by colour. */
  function scopeTag(name) {
    const text = String(name ?? '');
    const names = (state.shell ? state.shell.scopes : []).map(item => item.name);
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

  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));

  /* Times the band and the clock read out. */
  function when(at) {
    return new Date(at * 1000).toLocaleString(state.lang === 'ja' ? 'ja-JP' : 'en-GB', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
  }
  const day = at => new Date(at * 1000).toLocaleDateString(state.lang === 'ja' ? 'ja-JP' : 'en-GB');
  function tickAt(seq) {
    const ticks = state.band ? state.band.ticks : [];
    let found = null;
    for (const tick of ticks) {
      if (tick.seq > seq) break;
      found = tick;
    }
    return found;
  }

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

  const only = () => {
    const scope = state.scope;
    return scope && scope !== 'all' ? `?scope=${encodeURIComponent(scope)}` : '';
  };
  const PATHS = { shell: () => `/api/shell${only()}`, now: () => `/api/now${only()}`, band: () => '/api/ticks' };

  /* One read per wanted thing that is missing; dataArrived fills the state. */
  async function ensure(kind) {
    if (state[kind] || state.pending[kind] || !PATHS[kind]) return;
    state = { ...state, pending: { ...state.pending, [kind]: true } };
    try {
      const body = await (await read(PATHS[kind]())).json();
      state = window.GyState.apply(state, { type: 'dataArrived', kind, body: filled(kind, body) });
    } catch {
      state = window.GyState.apply(state, { type: 'dataArrived', kind, body: null });
    }
  }

  /* The band comes as { max, ticks }; the left end is the first tick's sequence. */
  function filled(kind, body) {
    if (kind !== 'band') return body;
    return { ...body, min: body.ticks.length ? body.ticks[0].seq : 1 };
  }

  async function reload() {
    await Promise.all([ensure('shell'), ensure('now'), ensure('band')]);
    paint();
  }

  /* Live updates: the band always follows the log; the rest only at the head. */
  async function moved() {
    state = { ...state, band: null };
    if (state.at === null) state = window.GyState.apply(state, { type: 'ledgerMoved' });
    await reload();
  }

  /* Draw the regions in order, then hand #main to the route's page. */
  function paint() {
    document.documentElement.lang = state.lang;
    if (window.GySidebar) window.GySidebar.render(state, document.querySelector('.side'), ui);
    if (window.GyTopbar) window.GyTopbar.render(state, document.getElementById('crumb'), ui);
    if (window.GyBand) window.GyBand.render(state, document.querySelector('.scrub'), ui);
    const label = document.getElementById('searchlbl');
    if (label) label.textContent = word('searchLbl'); /* temporary: palette's (n-45ca) */
    paintPage();
    if (window.GyRail) window.GyRail.draw(); /* temporary: rail region is n-45ca */
  }

  function paintPage() {
    const main = document.getElementById('main');
    main.className = '';
    const name = state.route.name;
    const arg = state.route.arg;
    if (name === 'now' && window.GyNow) window.GyNow.draw();
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
    if (intent.type === 'setScope' || intent.type === 'setAt') {
      if (intent.type === 'setAt') {
        preview();
        scheduleReload();
        return;
      }
      await reload();
    } else {
      paint();
    }
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

  /* Events: one delegated click and one keydown for the whole app. */
  function wire() {
    document.addEventListener('click', event => {
      const el = event.target.closest('[data-act]');
      if (!el) return;
      const act = el.dataset.act;
      const arg = el.dataset.arg ?? null;
      if (act === 'setScope') run({ type: 'setScope', value: arg });
      else if (act === 'setLang') run({ type: 'setLang', value: arg });
      else if (act === 'setAt') setAt(arg === 'now' ? null : Number(arg));
    });
    document.addEventListener('keydown', event => {
      if (composing(event) || event.target.tagName === 'INPUT') return;
      const head = state.at === null ? (state.band ? state.band.max : 0) : state.at;
      if (event.key === 'ArrowLeft') setAt(head - 1);
      if (event.key === 'ArrowRight') setAt(head + 1);
    });
    document.addEventListener('compositionend', () => { composedAt = performance.now(); });
    wireTrack();
    window.addEventListener('hashchange', () => go(window.GyState.parseRoute(location.hash)));
    window.addEventListener('resize', () => { if (window.GyBand) window.GyBand.render(state, document.querySelector('.scrub'), ui); });
    WIDE.addEventListener('change', () => run({ type: 'setWide', value: WIDE.matches }));
  }

  /* The band's drag: the root turns a pointer position into a sequence. */
  let dragging = false;
  function wireTrack() {
    const track = document.querySelector('#track');
    if (!track) return;
    const seqFrom = event => {
      const box = track.getBoundingClientRect();
      const band = state.band;
      return band.min + ((event.clientX - box.left) / box.width) * (band.max - band.min);
    };
    track.addEventListener('pointerdown', event => {
      dragging = true;
      track.setPointerCapture(event.pointerId);
      if (state.band) setAt(seqFrom(event));
    });
    track.addEventListener('pointermove', event => { if (dragging && state.band) setAt(seqFrom(event)); });
    track.addEventListener('pointerup', () => { dragging = false; });
  }

  /* What the regions may use; read-only helpers, not a way back into root. */
  const ui = { t: word, plural, scopeTag, tickAt, when, day };

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
    reload,
  };

  async function start() {
    const res = await fetch('/assets/i18n.json');
    words = await res.json();
    wire();
    await reload();
  }

  window.GyRoot = { moved };
  start();
})();