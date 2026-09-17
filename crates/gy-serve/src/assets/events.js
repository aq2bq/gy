/* The app's events (d-03ca, 第 2 段, split out of root): one delegated click,
   one keydown, the input, the hash, the resize, the band's drag, and the
   graph's pointer and wheel (第 3 段). It holds no state; it turns what happened
   into the root's calls, and the geometry it needs is the graph's pure
   functions (n-88b2). */
(function () {
  function wire(api) {
    document.addEventListener('click', event => click(api, event));
    document.addEventListener('input', event => {
      if (event.target.id === 'palq') api.run({ type: 'paletteQuery', value: event.target.value });
    });
    document.addEventListener('change', event => {
      if (event.target.id === 'hist-date') api.run({ type: 'historySince', value: window.GyHistory.since(api.state(), event.target.value) });
    });
    document.addEventListener('keydown', event => keys(api, event));
    document.addEventListener('compositionend', () => api.composedEnd());
    drag(api);
    graph(api);
    window.addEventListener('hashchange', () => api.go(window.GyState.parseRoute(location.hash)));
    window.addEventListener('resize', () => api.resized());
  }

  /* One click anywhere: the palette's frame, the sky, the graph's crumb and
     panel, then the plain acts the regions carry. */
  function click(api, event) {
    if (event.target.id === 'pal') { api.run({ type: 'paletteClose' }); return; }
    if (event.target.id === 'sky') {
      const id = window.GySky.hit(api.state(), event.target, event.clientX, event.clientY);
      if (id) location.hash = `#/graph/${id}`;
      return;
    }
    const crumb = event.target.closest('#gcrumb button');
    if (crumb) { graphCrumb(api, crumb); return; }
    const link = event.target.closest('#gpanel a[data-go]');
    if (link) { event.preventDefault(); graphPick(api, link.dataset.go, true); return; }
    const el = event.target.closest('[data-act]');
    if (!el) return;
    const act = el.dataset.act;
    const arg = el.dataset.arg ?? null;
    if (act === 'setScope') api.run({ type: 'setScope', value: arg });
    else if (act === 'setLang') api.run({ type: 'setLang', value: arg });
    else if (act === 'setAt') api.setAt(arg === 'now' ? null : Number(arg));
    else if (act === 'listFilter') api.run({ type: 'listFilter', value: arg });
    else if (act === 'historyActor') api.run({ type: 'historyActor', value: arg || null });
    else if (act === 'copy') copy(api, arg);
    else if (act === 'paletteGo') goHit(api, Number(arg));
  }

  /* One key: the palette's own keys first, then the band's arrows and the
     graph's two zoom keys. */
  function keys(api, event) {
    if (api.composing(event)) return;
    if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
      event.preventDefault();
      api.run({ type: 'paletteOpen' });
      return;
    }
    const typing = event.target.tagName === 'INPUT' || event.target.tagName === 'TEXTAREA';
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
    const canvas = document.getElementById('g');
    if (!canvas || typing || event.metaKey || event.ctrlKey || event.altKey) return;
    if (event.key === '+' || event.key === '=') {
      event.preventDefault();
      api.run({ type: 'graphCam', value: window.GyGraph.zoomAt(state, canvas, 1.4) });
    } else if (event.key === '-' || event.key === '_') {
      event.preventDefault();
      api.run({ type: 'graphCam', value: window.GyGraph.zoomAt(state, canvas, 1 / 1.4) });
    }
  }

  /* The copy mark: write the text, then let the state show the sign (n-6afd). No
     clipboard, no sign: a mark that does nothing is worse than no mark. */
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

  /* The graph's pointer and wheel: the canvas keeps the pointer while a drag is
     on; a still pointer over a node changes the hover; the geometry is pure. */
  function graph(api) {
    let down = null, clicked = -Infinity;
    document.addEventListener('pointerdown', event => {
      if (event.target.id !== 'g') return;
      down = { x: event.clientX, y: event.clientY, moved: false };
      event.target.setPointerCapture(event.pointerId);
      event.target.classList.add('grab');
    });
    document.addEventListener('pointermove', event => {
      const canvas = document.getElementById('g');
      if (!canvas) return;
      if (down) {
        const dx = event.clientX - down.x, dy = event.clientY - down.y;
        if (Math.hypot(dx, dy) > 3) down.moved = true;
        if (down.moved) {
          down.x = event.clientX; down.y = event.clientY;
          api.run({ type: 'graphCam', value: window.GyGraph.panBy(api.state(), dx, dy) });
        }
        return;
      }
      if (event.target.id !== 'g') return;
      const id = window.GyGraph.hit(api.state(), canvas, event.clientX, event.clientY);
      if (id !== api.state().graph.hover) api.run({ type: 'graphHover', value: id });
    });
    document.addEventListener('pointerup', event => {
      if (!down) return;
      const wasDrag = down.moved;
      down = null;
      const canvas = document.getElementById('g');
      if (canvas) canvas.classList.remove('grab');
      if (wasDrag || event.target.id !== 'g') return;
      clicked = graphClick(api, event.target, event, clicked);
    });
    document.addEventListener('wheel', event => wheelZoom(api, event), { passive: false });
  }

  /* The wheel over the canvas: one notch is one zoom about the pointer. */
  function wheelZoom(api, event) {
    if (event.target.id !== 'g') return;
    event.preventDefault();
    /* A line is 16 px and a page 400, so the same gesture moves the same way. */
    const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? 400 : 1;
    const factor = event.ctrlKey ? 0.01 : 0.0045;
    const ratio = Math.max(0.5, Math.min(2, Math.exp(-event.deltaY * unit * factor)));
    api.run({ type: 'graphCam', value: window.GyGraph.zoomAt(api.state(), event.target, ratio, event.clientX, event.clientY) });
  }

  /* One click on the canvas: a node, a bubble, or nothing. The second click on
     the same node opens its page (n-7c5d). Returns when it landed. */
  function graphClick(api, canvas, event, clicked) {
    const state = api.state();
    const id = window.GyGraph.hit(state, canvas, event.clientX, event.clientY);
    const now = performance.now();
    const double = now - clicked < 350;
    if (id) {
      if (double && id === state.graph.selected) location.hash = `#/n/${id}`;
      else graphPick(api, id, false);
      return now;
    }
    if (double) {
      api.run({ type: 'graphCam', value: window.GyGraph.zoomAt(state, canvas, 2, event.clientX, event.clientY) });
      return now;
    }
    const bubble = window.GyGraph.bubbleAt(state, canvas, event.clientX, event.clientY);
    if (bubble && state.graph.cam.k < 1.2) graphBubble(api, bubble);
    else api.run({ type: 'graphSelect', value: null });
    return now;
  }

  /* A node picked: the panel opens, and a close camera goes to it. */
  function graphPick(api, id, force) {
    const state = api.state();
    api.run({ type: 'graphSelect', value: id });
    if (id && (force || state.graph.cam.k < 2.2)) graphFit(api, { node: id });
  }

  /* A bubble picked: the camera goes to its scope. */
  function graphBubble(api, bubble) {
    api.run({ type: 'graphSelect', value: null });
    graphFit(api, { scope: bubble.scope });
  }

  function graphCrumb(api, button) {
    api.run({ type: 'graphSelect', value: null });
    graphFit(api, button.id === 'gback' ? null : { scope: button.dataset.scope });
  }

  /* The pure fit becomes the root's target; the root's clock moves the camera. */
  function graphFit(api, target) {
    const canvas = document.getElementById('g');
    if (!canvas) return;
    const cam = window.GyGraph.fitTo(api.state(), canvas, target);
    if (cam) api.run({ type: 'graphTarget', value: cam });
  }

  window.GyEvents = { wire };
})();