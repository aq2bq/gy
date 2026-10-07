/* The app's events (d-03ca, 第 2 段, split out of root): one delegated click,
   one keydown, the input, the hash, the resize, the band's drag, and the
   graph's pointer and wheel (第 3 段). It holds no state; it turns what happened
   into the root's calls, and the geometry it needs is the graph's pure
   functions (n-88b2).

   One event is two halves (r-bb59): a choice turns the state and what happened
   into the calls as data, and perform is the only place that runs them. The
   calls the last event chose are kept for what reads the screen. */
(function () {
  /* The calls the last event chose; read from the outside (r-bb59). */
  let last = [];

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
    const calls = clickPlan(api.state(), event);
    last = calls;
    perform(api, calls, event);
  }

  /* What one click does, from the state and where it landed. Pure: the event
     and the state in, the calls as data. */
  function clickPlan(state, event) {
    const target = event.target;
    if (target.id === 'pal') return [{ type: 'run', value: { type: 'paletteClose' } }];
    if (target.id === 'sky') {
      const id = window.GySky.hit(state, target, event.clientX, event.clientY);
      return id ? [{ type: 'hash', value: `#/graph/${id}` }] : [];
    }
    const crumb = target.closest('#gcrumb button');
    if (crumb) return [{ type: 'crumb', id: crumb.id, scope: crumb.dataset.scope }];
    const link = target.closest('#gpanel a[data-go]');
    if (link) return [{ type: 'prevent' }, { type: 'pick', id: link.dataset.go, force: true }];
    const el = target.closest('[data-act]');
    return el ? actPlan(el.dataset.act, el.dataset.arg ?? null) : [];
  }

  /* The plain acts a region carries (`data-act`), as calls. */
  function actPlan(act, arg) {
    switch (act) {
      case 'setScope': return [{ type: 'run', value: { type: 'setScope', value: arg } }];
      case 'setLang': return [{ type: 'run', value: { type: 'setLang', value: arg } }];
      case 'setAt': return [{ type: 'at', value: arg === 'now' ? null : Number(arg) }];
      case 'listFilter': return [{ type: 'run', value: { type: 'listFilter', value: arg } }];
      case 'historyActor': return [{ type: 'run', value: { type: 'historyActor', value: arg || null } }];
      case 'copy': return [{ type: 'clipboard', text: arg }];
      case 'paletteOpen': return [{ type: 'run', value: { type: 'paletteOpen' } }];
      case 'paletteGo': return [{ type: 'hit', index: Number(arg) }];
      default: return [];
    }
  }

  /* One key: the palette's own keys first, then the band's arrows and the two
     zoom keys of whichever figure is on the page (n-9ca9). */
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
    const head = state.at === null ? (state.band ? state.band.max : 0) : state.at;
    if (key === 'ArrowLeft') return [{ type: 'at', value: head - 1 }];
    if (key === 'ArrowRight') return [{ type: 'at', value: head + 1 }];
    return zoomPlan(event, typing);
  }

  /* The two zoom keys, for whichever figure is on the page (n-9ca9). */
  function zoomPlan(event, typing) {
    const canvas = document.getElementById('g');
    const map = document.getElementById('ego');
    if ((!canvas && !map) || typing || event.metaKey || event.ctrlKey || event.altKey) return [];
    const key = event.key;
    if (key !== '+' && key !== '=' && key !== '-' && key !== '_') return [];
    const ratio = key === '+' || key === '=' ? 1.4 : 1 / 1.4;
    return [{ type: 'prevent' }, { type: 'zoom', ratio, onCanvas: !!canvas }];
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

  /* The graph's pointer and wheel, and the node map's own: the canvas keeps the
     pointer while a drag is on; a still pointer over a node changes the hover;
     the geometry is pure. */
  function graph(api) {
    document.addEventListener('wheel', event => wheelZoom(api, event), { passive: false });
    graphPointer(api);
    mapPointer(api);
  }

  function graphPointer(api) {
    let down = null, clicked = -Infinity;
    document.addEventListener('pointerdown', event => {
      const plan = graphDown(event);
      if (!plan) return;
      down = plan.down;
      last = plan.calls;
      perform(api, plan.calls, event);
    });
    document.addEventListener('pointermove', event => {
      const plan = graphMove(api.state(), event, down);
      if (!plan) return;
      down = plan.down;
      last = plan.calls;
      perform(api, plan.calls, event);
    });
    document.addEventListener('pointerup', event => {
      const plan = graphUp(api.state(), event, down, clicked, performance.now());
      if (!plan) return;
      down = plan.down;
      clicked = plan.clicked;
      last = plan.calls;
      perform(api, plan.calls, event);
    });
  }

  /* A press on the canvas starts a drag, keeping the pointer (r-bb59). */
  function graphDown(event) {
    if (event.target.id !== 'g') return null;
    return {
      down: { x: event.clientX, y: event.clientY, moved: false },
      calls: [{ type: 'capture', element: event.target, pointerId: event.pointerId }, { type: 'grab', element: event.target, on: true }],
    };
  }

  /* A move: a drag pans it and hides the card; a still pointer changes the
     hover. Null when the graph is not on the page: the event is another
     figure's. Pure: the session in, the session and the calls out. */
  function graphMove(state, event, down) {
    const canvas = document.getElementById('g');
    if (!canvas) return null;
    if (!down) return { down: null, calls: hoverCalls(state, canvas, event) };
    const hover = state.graph.hover ? { type: 'run', value: { type: 'graphHover', value: null } } : null;
    return dragMove(down, event, (dx, dy) => ({ type: 'graphPan', dx, dy }), hover);
  }

  /* The node under a still pointer, and the card that follows it; off the
     canvas is off the node (d-1c00). */
  function hoverCalls(state, canvas, event) {
    if (event.target.id !== 'g') return state.graph.hover ? [{ type: 'run', value: { type: 'graphHover', value: null } }] : [];
    const id = window.GyGraph.hit(state, canvas, event.clientX, event.clientY);
    return id !== state.graph.hover ? [{ type: 'run', value: { type: 'graphHover', value: id } }] : [];
  }

  /* A drag's move, for the canvas and the map: past 3 px it pans by the step and
     hides the card (d-1c00, n-dc1a). */
  function dragMove(down, event, pan, hover) {
    const dx = event.clientX - down.x, dy = event.clientY - down.y;
    const moved = down.moved || Math.hypot(dx, dy) > 3;
    if (!moved) return { down, calls: [] };
    const calls = [pan(dx, dy)];
    if (hover) calls.push(hover);
    return { down: { x: event.clientX, y: event.clientY, moved: true }, calls };
  }

  /* A release: a drag only lets the grab go; a plain press on the canvas picks
     what it landed on. Null when there was no press on the canvas. Pure: when
     it landed and the calls as data. */
  function graphUp(state, event, down, clicked, now) {
    if (!down) return null;
    const canvas = document.getElementById('g');
    const calls = canvas ? [{ type: 'grab', element: canvas, on: false }] : [];
    if (down.moved || event.target.id !== 'g') return { down: null, clicked, calls };
    const result = graphClickPlan(state, event, clicked, now);
    return { down: null, clicked: result.clicked, calls: calls.concat(result.calls) };
  }

  /* One click on the canvas: a node, a bubble, or nothing. The second click on
     the same node opens its page (n-7c5d). Pure: when it landed and the calls. */
  function graphClickPlan(state, event, clicked, now) {
    const canvas = document.getElementById('g');
    const id = window.GyGraph.hit(state, canvas, event.clientX, event.clientY);
    const double = now - clicked < 350;
    if (id) {
      const calls = double && id === state.graph.selected
        ? [{ type: 'hash', value: `#/n/${id}` }]
        : [{ type: 'pick', id, force: false }];
      return { clicked: now, calls };
    }
    if (double) return { clicked: now, calls: [{ type: 'zoom', ratio: 2, x: event.clientX, y: event.clientY, onCanvas: true }] };
    const bubble = window.GyGraph.bubbleAt(state, canvas, event.clientX, event.clientY);
    if (bubble && state.graph.cam.k < 1.2) return { clicked: now, calls: [{ type: 'bubble', bubble }] };
    return { clicked: now, calls: [{ type: 'run', value: { type: 'graphSelect', value: null } }] };
  }

  /* The node map's camera: the pointer and the drag arrive in screen px, so the
     svg's box turns them into the units the map is drawn in; the pure math is
     the graph's, shared (n-9ca9). */
  function mapScale(svg) {
    const box = svg.getBoundingClientRect();
    const view = svg.viewBox.baseVal;
    return { sx: view.width / box.width, sy: view.height / box.height, box };
  }

  function mapZoom(api, svg, ratio, clientX, clientY) {
    const { sx, sy, box } = mapScale(svg);
    const px = clientX === undefined ? (box.width / 2) * sx : (clientX - box.left) * sx;
    const py = clientY === undefined ? (box.height / 2) * sy : (clientY - box.top) * sy;
    api.run({ type: 'mapCam', value: window.GyGraph.zoomCam(api.state().ego.cam, ratio, px, py) });
  }

  function mapPan(api, svg, dx, dy) {
    const { sx, sy } = mapScale(svg);
    api.run({ type: 'mapCam', value: window.GyGraph.panCam(api.state().ego.cam, dx * sx, dy * sy) });
  }

  /* The box under the pointer, by the name the region gave it (`data-node`):
     the box's <g> for any of its parts, and nothing off the map (n-dc1a). */
  function boxUnder(svg, target) {
    if (!svg.contains(target) || !target.closest) return null;
    const box = target.closest('[data-node]');
    return box ? box.dataset.node : null;
  }

  /* The node map's pointer: a drag pans it; a still pointer picks the box
     under it for the card. No capture, so a plain press still reaches a box's
     anchor (n-9ca9). */
  function mapPointer(api) {
    let down = null, swallow = false;
    /* A drag that began on a box would otherwise let the press through as a
       click and follow the anchor; consume that one click (n-9ca9). */
    document.addEventListener('click', event => {
      if (!swallow) return;
      swallow = false;
      event.preventDefault();
    }, true);
    document.addEventListener('pointerdown', event => {
      const plan = mapDown(event);
      swallow = false;
      if (!plan) return;
      down = plan.down;
      last = plan.calls;
      perform(api, plan.calls, event);
    });
    document.addEventListener('pointermove', event => {
      const plan = mapMove(api.state(), event, down);
      if (!plan) return;
      down = plan.down;
      last = plan.calls;
      perform(api, plan.calls, event);
    });
    document.addEventListener('pointerup', () => {
      const plan = mapUp(down);
      down = null;
      swallow = plan ? plan.swallow : false;
      if (!plan) return;
      last = plan.calls;
      perform(api, plan.calls, null);
    });
  }

  /* A press on the map starts a drag, with no capture (n-9ca9). */
  function mapDown(event) {
    const svg = document.getElementById('ego');
    if (!svg || !svg.contains(event.target)) return null;
    return { down: { x: event.clientX, y: event.clientY, moved: false }, calls: [{ type: 'grab', element: svg, on: true }] };
  }

  /* A move: a drag pans it; a still pointer picks the box under it. Null when
     the map is not on the page: the event is another figure's. */
  function mapMove(state, event, down) {
    const svg = document.getElementById('ego');
    if (!svg) return null;
    if (!down) {
      const id = boxUnder(svg, event.target);
      return { down: null, calls: id !== state.ego.hover ? [{ type: 'run', value: { type: 'mapHover', value: id } }] : [] };
    }
    const hover = state.ego.hover ? { type: 'run', value: { type: 'mapHover', value: null } } : null;
    return dragMove(down, event, (dx, dy) => ({ type: 'panMap', dx, dy }), hover);
  }

  /* A release: the grab goes, and a drag consumes the click that follows it.
     Null when the map is not on the page. */
  function mapUp(down) {
    const svg = document.getElementById('ego');
    if (!svg) return null;
    return { swallow: !!down && down.moved, calls: [{ type: 'grab', element: svg, on: false }] };
  }

  /* The wheel over a figure: one notch is one zoom about the pointer, on the
     graph's canvas or the node's map (n-9ca9). */
  function wheelZoom(api, event) {
    const canvas = document.getElementById('g');
    const map = document.getElementById('ego');
    const onCanvas = canvas && event.target.id === 'g';
    const onMap = map && map.contains(event.target);
    if (!onCanvas && !onMap) return;
    event.preventDefault();
    /* A line is 16 px and a page 400, so the same gesture moves the same way. */
    const unit = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? 400 : 1;
    const factor = event.ctrlKey ? 0.01 : 0.0045;
    const ratio = Math.max(0.5, Math.min(2, Math.exp(-event.deltaY * unit * factor)));
    const state = api.state();
    if (onCanvas) {
      const value = window.GyGraph.zoomAt(state, canvas, ratio, event.clientX, event.clientY);
      api.run({ type: 'graphCam', value });
      return;
    }
    mapZoom(api, map, ratio, event.clientX, event.clientY);
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

  function graphCrumb(api, id, scope) {
    api.run({ type: 'graphSelect', value: null });
    graphFit(api, id === 'gback' ? null : { scope });
  }

  /* The pure fit becomes the root's target; the root's clock moves the camera. */
  function graphFit(api, target) {
    const canvas = document.getElementById('g');
    if (!canvas) return;
    const cam = window.GyGraph.fitTo(api.state(), canvas, target);
    if (cam) api.run({ type: 'graphTarget', value: cam });
  }

  /* The one zoom, for the canvas or the map, about a point when given. */
  function zoomCall(api, step) {
    const state = api.state();
    if (step.onCanvas) {
      api.run({ type: 'graphCam', value: window.GyGraph.zoomAt(state, document.getElementById('g'), step.ratio, step.x, step.y) });
      return;
    }
    mapZoom(api, document.getElementById('ego'), step.ratio, step.x, step.y);
  }

  /* The decided calls, becoming acts: the root, the hash, and the DOM. Nothing
     is judged here (r-bb59). */
  function perform(api, calls, event) {
    for (const step of calls) {
      const go = CALLS[step.type];
      if (go) go(api, step, event);
    }
  }

  /* One name per call: what each decided call does. */
  const CALLS = {
    run: (api, step) => api.run(step.value),
    at: (api, step) => api.setAt(step.value),
    hash: (api, step) => { location.hash = step.value; },
    prevent: (api, step, event) => event.preventDefault(),
    capture: (api, step) => step.element.setPointerCapture(step.pointerId),
    grab: (api, step) => step.element.classList.toggle('grab', step.on),
    pick: (api, step) => graphPick(api, step.id, step.force),
    bubble: (api, step) => graphBubble(api, step.bubble),
    crumb: (api, step) => graphCrumb(api, step.id, step.scope),
    hit: (api, step) => goHit(api, step.index),
    clipboard: (api, step) => copy(api, step.text),
    palette: (api, step, event) => paletteKey(api, event),
    zoom: (api, step) => zoomCall(api, step),
    graphPan: (api, step) => api.run({ type: 'graphCam', value: window.GyGraph.panBy(api.state(), step.dx, step.dy) }),
    panMap: (api, step) => mapPan(api, document.getElementById('ego'), step.dx, step.dy),
  };

  window.GyEvents = { wire, lastCalls: () => last };
})();