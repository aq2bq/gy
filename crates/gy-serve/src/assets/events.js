/* The app's events (d-03ca, split out of root): one delegated click, one
   keydown, the palette's input, the history's day, the hash, the resize, and
   the band's drag. It holds no state; it turns what happened into the root's
   calls. One event is two halves (r-bb59): a choice turns the state and what
   happened into the calls as data, and perform is the only place that runs
   them. */
(function () {
  /* The calls the last event chose; read from the outside (r-bb59). */
  let last = [];

  function wire(api) {
    document.addEventListener('click', event => click(api, event));
    document.addEventListener('input', event => {
      if (event.target.id === 'palq') api.run({ type: 'paletteQuery', value: event.target.value });
    });
    document.addEventListener('change', event => {
      if (event.target.id === 'hist-date') {
        api.run({ type: 'historySince', value: window.GyHistory.since(api.state(), event.target.value) });
      }
    });
    document.addEventListener('keydown', event => keys(api, event));
    document.addEventListener('compositionend', () => api.composedEnd());
    drag(api);
    window.addEventListener('hashchange', () => api.go(window.GyState.parseRoute(location.hash)));
    window.addEventListener('resize', () => api.resized());
  }

  /* One click anywhere: the palette's frame first, then the acts a region
     carries (`data-act`). A plain anchor is the browser's. */
  function click(api, event) {
    const calls = clickPlan(api.state(), event);
    last = calls;
    perform(api, calls, event);
  }

  /* What one click does, from the state and where it landed. Pure: the event
     and the state in, the calls as data. */
  function clickPlan(state, event) {
    const target = event.target;
    if (target.id === 'pal') return [{ type: 'run', value: { type: 'paletteClose' } }];
    const el = target.closest('[data-act]');
    return el ? actPlan(el.dataset.act, el.dataset.arg ?? null) : [];
  }

  /* The plain acts a region carries (`data-act`), as calls. */
  function actPlan(act, arg) {
    switch (act) {
      case 'setLang': return [{ type: 'run', value: { type: 'setLang', value: arg } }];
      case 'setScope': return [{ type: 'run', value: { type: 'setScope', value: arg } }];
      case 'setAt': return [{ type: 'at', value: arg === 'now' ? null : Number(arg) }];
      case 'listFilter': return [{ type: 'run', value: { type: 'listFilter', value: arg } }];
      case 'historyActor': return [{ type: 'run', value: { type: 'historyActor', value: arg || null } }];
      case 'copy': return [{ type: 'clipboard', text: arg }];
      case 'paletteOpen': return [{ type: 'run', value: { type: 'paletteOpen' } }];
      case 'paletteGo': return [{ type: 'hit', index: Number(arg) }];
      case 'detailDepth': return [{ type: 'run', value: { type: 'detailDepth', value: Number(arg) } }];
      case 'retry': return [{ type: 'retry' }];
      default: return [];
    }
  }

  /* One key: the palette's own keys first, then the band's arrows. */
  function keys(api, event) {
    if (api.composing(event)) return;
    const calls = keyPlan(api.state(), event);
    last = calls;
    perform(api, calls, event);
  }

  /* What one key does. Pure: the event and the state in, the calls as data. */
  function keyPlan(state, event) {
    const key = event.key;
    if ((event.metaKey || event.ctrlKey) && key.toLowerCase() === 'k') return [{ type: 'prevent' }, { type: 'run', value: { type: 'paletteOpen' } }];
    const typing = event.target.tagName === 'INPUT' || event.target.tagName === 'TEXTAREA';
    if (state.palette.open && event.target.id === 'palq') return [{ type: 'palette' }];
    if (key === '/' && !typing) return [{ type: 'prevent' }, { type: 'run', value: { type: 'paletteOpen' } }];
    if (typing) return [];
    const head = state.at === null ? (state.band ? state.band.max : 0) : state.at;
    if (key === 'ArrowLeft') return [{ type: 'at', value: head - 1 }];
    if (key === 'ArrowRight') return [{ type: 'at', value: head + 1 }];
    return [];
  }

  /* The copy mark: write the text, then let the state show the sign (n-6afd).
     No clipboard, no sign: a mark that does nothing is worse than no mark. */
  function copy(api, text) {
    if (!navigator.clipboard) return;
    navigator.clipboard.writeText(text).then(
      () => api.run({ type: 'copied', value: text }),
      () => {},
    );
  }

  /* The palette's own keys, while its input has the focus. */
  function paletteKey(api, event) {
    const intent = window.GyPalette.key(api.state(), event);
    if (!intent) return;
    if (intent.prevent) event.preventDefault();
    if (intent.type === 'paletteEnter') {
      if (api.composedRecently()) return;
      goHit(api, api.state().palette.selected);
      return;
    }
    api.run(intent);
  }

  function goHit(api, index) {
    const state = api.state();
    const hits = state.search ? state.search.hits || [] : [];
    const hit = hits[index];
    if (!hit) return;
    location.hash = `#/n/${hit.id}`;
  }

  /* The band's drag: a pointer position becomes a sequence. */
  function drag(api) {
    const scrub = document.querySelector('.scrub');
    const track = scrub && scrub.querySelector('#track');
    if (!track) return;
    let down = false;
    track.addEventListener('pointerdown', event => {
      down = true;
      track.setPointerCapture(event.pointerId);
      if (api.state().band) api.setAt(window.GyBand.seqFrom(api.state(), scrub, event.clientX));
    });
    track.addEventListener('pointermove', event => { if (down && api.state().band) api.setAt(window.GyBand.seqFrom(api.state(), scrub, event.clientX)); });
    track.addEventListener('pointerup', () => { down = false; });
  }

  /* The decided calls, becoming acts: the root, the hash, and the DOM. */
  function perform(api, calls, event) {
    for (const step of calls) {
      const go = CALLS[step.type];
      if (go) go(api, step, event);
    }
  }

  const CALLS = {
    run: (api, step) => api.run(step.value),
    at: (api, step) => api.setAt(step.value),
    hash: (api, step) => { location.hash = step.value; },
    prevent: (api, step, event) => event.preventDefault(),
    hit: (api, step) => goHit(api, step.index),
    clipboard: (api, step) => copy(api, step.text),
    retry: api => api.retry(),
    palette: (api, step, event) => paletteKey(api, event),
  };

  window.GyEvents = { wire, lastCalls: () => last };
})();
