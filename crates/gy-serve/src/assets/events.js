/* The app's events (d-03ca, 第 2 段, split out of root): one delegated click,
   one keydown, the input, the hash, the resize, and the band's drag. It holds
   no state; it turns what happened into the root's calls. */
(function () {
  function wire(api) {
    document.addEventListener('click', event => {
      if (event.target.id === 'pal') { api.run({ type: 'paletteClose' }); return; }
      if (event.target.id === 'sky') {
        const id = window.GySky.hit(api.state(), event.target, event.clientX, event.clientY);
        if (id) location.hash = `#/graph/${id}`;
        return;
      }
      const el = event.target.closest('[data-act]');
      if (!el) return;
      const act = el.dataset.act;
      const arg = el.dataset.arg ?? null;
      if (act === 'setScope') api.run({ type: 'setScope', value: arg });
      else if (act === 'setLang') api.run({ type: 'setLang', value: arg });
      else if (act === 'setAt') api.setAt(arg === 'now' ? null : Number(arg));
      else if (act === 'paletteGo') goHit(api, Number(arg));
    });
    document.addEventListener('input', event => {
      if (event.target.id === 'palq') api.run({ type: 'paletteQuery', value: event.target.value });
    });
    document.addEventListener('keydown', event => {
      if (api.composing(event)) return;
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        api.run({ type: 'paletteOpen' });
        return;
      }
      const typing = event.target.tagName === 'INPUT';
      if (api.state().palette.open && event.target.id === 'palq') { paletteKey(api, event); return; }
      if (event.key === '/' && !typing) {
        event.preventDefault();
        api.run({ type: 'paletteOpen' });
        return;
      }
      const state = api.state();
      const head = state.at === null ? (state.band ? state.band.max : 0) : state.at;
      if (event.key === 'ArrowLeft') api.setAt(head - 1);
      if (event.key === 'ArrowRight') api.setAt(head + 1);
    });
    document.addEventListener('compositionend', () => api.composedEnd());
    drag(api);
    window.addEventListener('hashchange', () => api.go(window.GyState.parseRoute(location.hash)));
    window.addEventListener('resize', () => api.resized());
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
    api.run({ type: 'paletteClose' });
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

  window.GyEvents = { wire };
})();