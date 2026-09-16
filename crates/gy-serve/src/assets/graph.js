/* The graph page (n-4cd8, a region since d-03ca 第 3 段): the picture, the
   crumb, and the panel. It draws from the state the root read; the pointer and
   the keys are heard in events.js, and the geometry those handlers need is pure
   below. The camera's ease is the root's clock (n-88b2). */
(function () {
  const KINDS = ['Need', 'Question', 'Decision', 'Requirement', 'Criterion'];
  const MIN_K = 0.15, MAX_K = 12;

  /* The last state drawn, for the two handles e2e reads (n-88b2). */
  let current = null;

  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));

  /* The view graph-draw.js draws, built from the one state. */
  function makeView(state) {
    return {
      graph: state.map,
      cam: state.graph.cam,
      selected: state.graph.selected,
      hover: state.graph.hover,
      labels: new Map(Object.entries(state.labels || {})),
    };
  }

  /* The scaffold, once per visit: the canvas, the crumb, the panel, the hint,
     and the legend. A language switch does not rebuild it (as before). */
  function build(el, ui) {
    el.innerHTML =
      `<div class="gwrap"><canvas id="g"></canvas><div class="gcrumb" id="gcrumb"></div><div class="gpanel" id="gpanel"></div>` +
      `<div class="ghint">${ui.t('gHint')}</div>` +
      `<div class="legend">${KINDS.map(kind => `<span><span class="dot dot-${kind}"></span>${ui.t(kind)}</span>`).join('')}</div></div>`;
  }

  /* The panel: the selected node's answer (state.page), as the old page drew it. */
  function panel(box, state, ui) {
    const id = state.graph.selected;
    const body = state.page;
    const node = (state.map.nodes || []).find(item => item.id === id);
    if (!id || !body || !node) {
      box.classList.remove('on');
      return;
    }
    const word = node.state ? ui.t(`st.${node.kind}.${node.state}`) : '';
    const relations = (body.edges || [])
      .map(edge => `<a href="#" data-go="${edge.to}"><span class="rl">${esc(ui.t(`rel.${edge.name}`))}</span>${esc(edge.alias || edge.to)} ${esc((edge.title || '').slice(0, 30))}</a>`)
      .join('');
    box.classList.add('on');
    box.innerHTML =
      `<div class="kk"><span class="k k-${node.kind}">${ui.t(node.kind)}</span><span>${esc((body.aliases || [])[0] || id)}</span>${ui.scopeTag(node.scope)}<span>${esc(word)}</span></div>` +
      `<div class="tt">${esc(body.title || '')}</div>` +
      `<div class="rel">${relations || `<span style="color:var(--paper-3)">${ui.t('noConn')}</span>`}</div>` +
      `<a class="open" href="#/n/${id}">${ui.t('gOpen')}</a>`;
  }

  const step = k => (k < 0.9 ? 'lod0' : k < 1.6 ? 'lod1' : k < 3.2 ? 'lod2' : 'lod3');

  /* The region: #main only, from the state the root read. */
  function render(state, el, ui) {
    if (!el) return;
    current = state;
    el.className = 'wide';
    if (!el.querySelector('#g')) build(el, ui);
    const c = el.querySelector('#g');
    if (!c || !state.map) return;
    const view = makeView(state);
    c.style.cursor = state.graph.hover ? 'pointer' : 'grab';
    const crumb = el.querySelector('#gcrumb');
    if (crumb) crumb.innerHTML = window.GyDraw.crumb(c, view, ui.t, step(view.cam.k));
    const box = el.querySelector('#gpanel');
    if (box) panel(box, state, ui);
    window.GyDraw.frame(c, view);
  }

  /* The node under a canvas point, if any. */
  function hit(state, c, clientX, clientY) {
    if (!state.map || !c) return null;
    const box = c.getBoundingClientRect();
    return window.GyDraw.pick(makeView(state), clientX - box.left, clientY - box.top);
  }

  /* The bubble under a canvas point, if any. */
  function bubbleAt(state, c, clientX, clientY) {
    if (!state.map || !c) return null;
    const box = c.getBoundingClientRect();
    return window.GyDraw.pickBubble(makeView(state), clientX - box.left, clientY - box.top);
  }

  /* The camera zoomed about a canvas point (the middle when none is given). */
  function zoomAt(state, c, ratio, clientX, clientY) {
    const cam = state.graph.cam;
    const box = c.getBoundingClientRect();
    const px = clientX === undefined ? box.width / 2 : clientX - box.left;
    const py = clientY === undefined ? box.height / 2 : clientY - box.top;
    const k = Math.max(MIN_K, Math.min(MAX_K, cam.k * ratio));
    const applied = k / cam.k;
    return { k, x: px - (px - cam.x) * applied, y: py - (py - cam.y) * applied };
  }

  function panBy(state, dx, dy) {
    return { k: state.graph.cam.k, x: state.graph.cam.x + dx, y: state.graph.cam.y + dy };
  }

  /* Where the camera goes to show a node or a scope; everything when neither. */
  function fitTo(state, c, target) {
    const graph = state.map;
    if (!c || !graph || !graph.bubbles.length) return null;
    let box = null;
    if (target && target.scope) {
      const bubble = graph.bubbles.find(item => item.scope === target.scope);
      if (bubble) box = { x0: bubble.x - bubble.r, y0: bubble.y - bubble.r, x1: bubble.x + bubble.r, y1: bubble.y + bubble.r };
    } else if (target && target.node) {
      const node = graph.nodes.find(item => item.id === target.node);
      if (node) box = { x0: node.x - 120, y0: node.y - 120, x1: node.x + 120, y1: node.y + 120 };
    }
    if (!box) {
      box = graph.bubbles.reduce((held, bubble) => ({
        x0: Math.min(held.x0, bubble.x - bubble.r),
        y0: Math.min(held.y0, bubble.y - bubble.r),
        x1: Math.max(held.x1, bubble.x + bubble.r),
        y1: Math.max(held.y1, bubble.y + bubble.r),
      }), { x0: 1e9, y0: 1e9, x1: -1e9, y1: -1e9 });
    }
    const w = c.clientWidth;
    const h = c.clientHeight - 120;
    const k = Math.max(MIN_K, Math.min(MAX_K, Math.min(w / (box.x1 - box.x0), h / (box.y1 - box.y0)) * 0.86));
    return { k, x: w / 2 - (box.x0 + box.x1) / 2 * k, y: h / 2 - (box.y0 + box.y1) / 2 * k };
  }

  /* The camera at `elapsed` ms into a fit (620 ms, cubic out); pure. The root
     owns the clock and the one rAF (n-88b2). */
  function eased(from, to, elapsed) {
    let p = Math.min(1, elapsed / 620);
    p = 1 - (1 - p) ** 3;
    return {
      k: Math.exp(Math.log(from.k) + (Math.log(to.k) - Math.log(from.k)) * p),
      x: from.x + (to.x - from.x) * p,
      y: from.y + (to.y - from.y) * p,
      done: p >= 1,
    };
  }

  window.GyGraph = {
    render, hit, bubbleAt, zoomAt, panBy, fitTo, eased,
    /* The test's two handles for e2e graph.spec.ts; they go when 第 4 段 moves
       e2e to testids and roles (n-88b2). */
    get cam() { return current ? current.graph.cam : null; },
    worldToScreen: (x, y) => window.GyDraw.span(makeView(current), x, y),
  };
})();