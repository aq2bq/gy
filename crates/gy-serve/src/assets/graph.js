/* The operations half of the graph page (n-4cd8, d-b93b): the state, the
   loads, the camera, the panel, and the controls. The canvas is in
   graph-draw.js. */
(function () {
  const KINDS = ['Need', 'Question', 'Decision', 'Requirement', 'Criterion'];
  const FIT_MS = 620;
  const OPEN = ['open', 'filed', 'approved', 'satisfied', 'unsatisfied'];

  const view = { graph: null, cam: { x: 0, y: 0, k: 1 }, selected: null, hover: null, labels: new Map() };
  let loaded = null, asked = new Set(), frame = null, anim = null, down = null, moved = false, clicked = 0;

  const t = key => window.GyShell.t(key);
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const canvas = () => document.getElementById('g');

  async function draw(focus) {
    if (!canvas()) build();
    const scope = window.GyShell.scope();
    const at = window.GyShell.at;
    if (!loaded || loaded.scope !== scope || loaded.at !== at) {
      const res = await window.GyShell.read(`/api/graph?scope=${encodeURIComponent(scope)}`);
      const body = await res.json();
      view.graph = { ...body, nodes: body.nodes.map(node => ({ ...node, open: node.state === null || OPEN.includes(node.state) })) };
      loaded = { scope, at };
      view.selected = null;
      view.hover = null;
      fitTo(null);
    }
    if (focus) {
      select(focus);
      fitTo({ node: focus });
      return;
    }
    if (view.selected) await select(view.selected);
    schedule();
  }

  function schedule() {
    if (frame) return;
    frame = requestAnimationFrame(() => {
      frame = null;
      paint();
    });
  }

  function paint() {
    const wanted = window.GyDraw.frame(view);
    const step = view.cam.k < 0.9 ? 'lod0' : view.cam.k < 1.6 ? 'lod1' : view.cam.k < 3.2 ? 'lod2' : 'lod3';
    const el = document.getElementById('gcrumb');
    if (el) el.innerHTML = window.GyDraw.crumb(view, t, step);
    if (wanted.length) loadLabels(wanted);
  }

  async function loadLabels(ids) {
    const missing = ids.filter(id => !asked.has(id));
    if (!missing.length) return;
    missing.forEach(id => asked.add(id));
    for (let at = 0; at < missing.length; at += 200) {
      const res = await window.GyShell.read(`/api/labels?ids=${encodeURIComponent(missing.slice(at, at + 200).join(','))}`);
      const body = await res.json();
      for (const [id, label] of Object.entries(body.labels || {})) view.labels.set(id, label);
    }
    schedule();
  }

  async function select(id) {
    view.selected = id;
    const panel = document.getElementById('gpanel');
    if (!panel) return;
    if (!id) {
      panel.classList.remove('on');
      schedule();
      return;
    }
    const node = view.graph.nodes.find(item => item.id === id);
    const res = await window.GyShell.read(`/api/node/${encodeURIComponent(id)}`);
    const body = res.ok ? await res.json() : {};
    const state = node && node.state ? t(`st.${node.kind}.${node.state}`) : '';
    const relations = (body.edges || [])
      .map(edge => `<a href="#" data-go="${edge.to}"><span class="rl">${esc(t(`rel.${edge.name}`))}</span>${esc(edge.alias || edge.to)} ${esc((edge.title || '').slice(0, 30))}</a>`)
      .join('');
    panel.classList.add('on');
    panel.innerHTML =
      `<div class="kk"><span class="k k-${node.kind}">${t(node.kind)}</span><span>${esc((body.aliases || [])[0] || id)}</span><span>${esc(node.scope)}</span><span>${esc(state)}</span></div>` +
      `<div class="tt">${esc(body.title || '')}</div>` +
      `<div class="rel">${relations || `<span style="color:var(--paper-3)">${t('noConn')}</span>`}</div>` +
      `<a class="open" href="#/n/${id}">${t('gOpen')}</a>`;
    schedule();
  }

  function fitTo(target) {
    const c = canvas();
    if (!c || !view.graph || !view.graph.bubbles.length) return;
    let box = null;
    if (target && target.scope) {
      const bubble = view.graph.bubbles.find(item => item.scope === target.scope);
      if (bubble) box = { x0: bubble.x - bubble.r, y0: bubble.y - bubble.r, x1: bubble.x + bubble.r, y1: bubble.y + bubble.r };
    } else if (target && target.node) {
      const node = view.graph.nodes.find(item => item.id === target.node);
      if (node) box = { x0: node.x - 120, y0: node.y - 120, x1: node.x + 120, y1: node.y + 120 };
    }
    if (!box) {
      box = view.graph.bubbles.reduce((held, bubble) => ({
        x0: Math.min(held.x0, bubble.x - bubble.r),
        y0: Math.min(held.y0, bubble.y - bubble.r),
        x1: Math.max(held.x1, bubble.x + bubble.r),
        y1: Math.max(held.y1, bubble.y + bubble.r),
      }), { x0: 1e9, y0: 1e9, x1: -1e9, y1: -1e9 });
    }
    const w = c.clientWidth;
    const h = c.clientHeight - 120;
    const k = Math.max(0.15, Math.min(12, Math.min(w / (box.x1 - box.x0), h / (box.y1 - box.y0)) * 0.86));
    ease({ k, x: w / 2 - (box.x0 + box.x1) / 2 * k, y: h / 2 - (box.y0 + box.y1) / 2 * k });
  }

  function ease(to) {
    const from = { ...view.cam };
    const start = performance.now();
    cancelAnimationFrame(anim);
    const step = now => {
      let p = Math.min(1, (now - start) / FIT_MS);
      p = 1 - (1 - p) ** 3;
      view.cam = { k: Math.exp(Math.log(from.k) + (Math.log(to.k) - Math.log(from.k)) * p), x: from.x + (to.x - from.x) * p, y: from.y + (to.y - from.y) * p };
      schedule();
      if (p < 1) anim = requestAnimationFrame(step);
    };
    anim = requestAnimationFrame(step);
  }

  function build() {
    const main = document.getElementById('main');
    main.className = 'wide';
    main.innerHTML =
      `<div class="gwrap"><canvas id="g"></canvas><div class="gcrumb" id="gcrumb"></div><div class="gpanel" id="gpanel"></div>` +
      `<div class="ghint">${t('gHint')}</div>` +
      `<div class="legend">${KINDS.map(kind => `<span><span class="dot dot-${kind}"></span>${t(kind)}</span>`).join('')}</div></div>`;
    const c = canvas();
    const point = event => {
      const box = c.getBoundingClientRect();
      return { x: event.clientX - box.left, y: event.clientY - box.top };
    };
    c.addEventListener('pointerdown', event => {
      down = { x: event.clientX, y: event.clientY, cx: view.cam.x, cy: view.cam.y };
      moved = false;
      c.setPointerCapture(event.pointerId);
      c.classList.add('grab');
    });
    c.addEventListener('pointermove', event => {
      if (down) {
        const dx = event.clientX - down.x;
        const dy = event.clientY - down.y;
        if (Math.hypot(dx, dy) > 3) moved = true;
        if (moved) {
          cancelAnimationFrame(anim);
          view.cam = { k: view.cam.k, x: down.cx + dx, y: down.cy + dy };
          schedule();
        }
        return;
      }
      const p = point(event);
      const found = window.GyDraw.pick(view, p.x, p.y);
      if (found !== view.hover) {
        view.hover = found;
        c.style.cursor = found ? 'pointer' : 'grab';
        schedule();
      }
    });
    c.addEventListener('pointerup', event => {
      c.classList.remove('grab');
      const wasDrag = moved;
      down = null;
      if (wasDrag) return;
      const p = point(event);
      const id = window.GyDraw.pick(view, p.x, p.y);
      const now = performance.now();
      const double = now - clicked < 350;
      clicked = now;
      if (id) {
        if (double && id === view.selected) {
          location.hash = `#/n/${id}`;
          return;
        }
        select(id);
        if (view.cam.k < 2.2) fitTo({ node: id });
        return;
      }
      const bubble = window.GyDraw.pickBubble(view, p.x, p.y);
      if (bubble && view.cam.k < 1.2) {
        select(null);
        fitTo({ scope: bubble.scope });
        return;
      }
      select(null);
    });
    c.addEventListener('wheel', event => {
      event.preventDefault();
      cancelAnimationFrame(anim);
      const p = point(event);
      const k = Math.max(0.15, Math.min(12, view.cam.k * Math.exp(-event.deltaY * 0.0015)));
      const ratio = k / view.cam.k;
      view.cam = { k, x: p.x - (p.x - view.cam.x) * ratio, y: p.y - (p.y - view.cam.y) * ratio };
      schedule();
    }, { passive: false });
    document.getElementById('gcrumb').addEventListener('click', event => {
      const button = event.target.closest('button');
      if (!button) return;
      select(null);
      fitTo(button.id === 'gback' ? null : { scope: button.dataset.scope });
    });
    document.getElementById('gpanel').addEventListener('click', event => {
      const link = event.target.closest('a[data-go]');
      if (!link) return;
      event.preventDefault();
      select(link.dataset.go);
      fitTo({ node: link.dataset.go });
    });
  }

  window.GyGraph = { draw, worldToScreen: (x, y) => window.GyDraw.span(view, x, y), get cam() { return view.cam; } };
})();
