/* The canvas half of the graph page (n-4cd8): one frame per call, in the
   prototype's order. The state lives in graph.js and arrives as `view`.
   One frame is two halves (d-8b47): plan turns the view and the screen size
   into a display list of what to draw, and draw only puts that list on the
   canvas. The list the last frame made is kept for what reads the screen. */
(function () {
  /* The scales where the edges, the ids, and the titles appear. */
  const EDGE_K = 0.9, ID_K = 1.6, TITLE_K = 3.2;
  let palette = null, last = null;

  /* The kind colours and the two fonts, read from the stylesheet once. */
  function colours() {
    if (palette) return palette;
    const style = getComputedStyle(document.body);
    const value = name => style.getPropertyValue(name).trim();
    palette = { Need: value('--need'), Question: value('--question'), Decision: value('--decision'), Requirement: value('--requirement'), Criterion: value('--criterion'), mono: value('--mono'), sans: value('--sans') };
    return palette;
  }

  const span = (view, x, y) => ({ x: x * view.cam.k + view.cam.x, y: y * view.cam.k + view.cam.y });
  const find = (view, id) => view.graph.nodes.find(node => node.id === id) || null;
  const alias = (view, id) => (view.labels.get(id) || {}).alias || id;

  /* The ids next to the selected node, itself included. */
  function neighbours(view) {
    const near = new Set([view.selected]);
    for (const [a, b] of [...view.graph.edges, ...view.graph.cross]) {
      if (a === view.selected) near.add(b);
      if (b === view.selected) near.add(a);
    }
    return near;
  }

  /* A bubble's radius, and a node's, as the frame has always drawn them. */
  function nodeRadius(node, k) {
    return Math.max(1.6, (2.2 + Math.sqrt(node.degree) * 1.3) * Math.min(k, 4));
  }
  function nodeAlpha(near, id) {
    return !near || near.has(id) ? 1 : 0.22;
  }

  /* A world box wholly off the canvas, with the frame's 40 px of slack. */
  function offscreen(A, B, w, h) {
    return (A.x < 0 && B.x < 0) || (A.y < 0 && B.y < 0) || (A.x > w && B.x > w) || (A.y > h && B.y > h);
  }

  /* The bubbles: a dark disc and the scope name (with its count when wide). */
  function bubblePlan(view, pal) {
    return view.graph.bubbles.map(bubble => {
      const p = span(view, bubble.x, bubble.y), r = bubble.r * view.cam.k;
      const wide = view.cam.k < EDGE_K;
      const size = Math.max(11, Math.min(22, 14 * view.cam.k));
      return {
        scope: bubble.scope,
        circle: { x: p.x, y: p.y, r, fill: 'rgba(28,36,56,.35)', stroke: 'rgba(42,52,80,.9)', lineWidth: 1 },
        text: { value: bubble.scope + (wide ? `  ${bubble.count}` : ''), x: p.x, y: p.y - r - 8, font: `${size}px ${pal.mono}`, align: 'center', color: wide ? 'rgba(236,230,216,.85)' : 'rgba(126,122,112,.9)' },
      };
    });
  }

  /* The edges between scopes, pale unless one end is near the selection. */
  function crossPlan(view, near, dim) {
    const lines = [];
    for (const [a, b] of view.graph.cross) {
      const from = find(view, a), to = find(view, b);
      if (!from || !to) continue;
      const hi = near && (near.has(a) || near.has(b));
      const A = span(view, from.x, from.y), B = span(view, to.x, to.y);
      lines.push({ from: a, to: b, x1: A.x, y1: A.y, x2: B.x, y2: B.y, lineWidth: Math.max(0.5, 0.8 * view.cam.k), stroke: hi ? 'rgba(236,230,216,.9)' : `rgba(95,211,199,${0.12 * dim})` });
    }
    return lines;
  }

  /* One edge inside a bubble, or null when it leaves the scope or the screen. */
  function edgePlan(view, w, h, near, dim, bubble, a, b) {
    const from = find(view, a), to = find(view, b);
    if (!from || !to || from.scope !== bubble.scope) return null;
    const A = span(view, from.x, from.y), B = span(view, to.x, to.y);
    if (offscreen(A, B, w, h)) return null;
    const hi = near && near.has(a) && near.has(b) && (a === view.selected || b === view.selected);
    return { from: a, to: b, x1: A.x, y1: A.y, x2: B.x, y2: B.y, stroke: hi ? 'rgba(236,230,216,.95)' : `rgba(185,179,166,${0.35 * dim})`, lineWidth: hi ? Math.max(1.2, 1.4 * view.cam.k) : Math.max(0.4, 0.7 * view.cam.k) };
  }

  /* The edges inside a bubble, once its disc is wide enough to read. */
  function edgesPlan(view, w, h, near, dim) {
    const lines = [];
    for (const bubble of view.graph.bubbles) {
      if (bubble.r * view.cam.k < 150) continue;
      for (const [a, b] of view.graph.edges) {
        const line = edgePlan(view, w, h, near, dim, bubble, a, b);
        if (line) lines.push(line);
      }
    }
    return lines;
  }

  /* The names beside a node: its id when the scale or the pointer asks, and its
     title under the same names. Empty while the frame shows only dots. */
  function textPlan(view, node, p, r, alpha, selected, hover, pal) {
    if (view.cam.k < ID_K && !selected && !hover) return [];
    const texts = [{ role: 'id', id: node.id, value: alias(view, node.id), x: p.x + r + 5, y: p.y + 3.5, font: `${selected ? 12 : 10}px ${pal.mono}`, align: 'left', color: 'rgba(236,230,216,.9)', alpha }];
    if (view.cam.k >= TITLE_K || selected || hover) {
      const title = (view.labels.get(node.id) || {}).title || '';
      texts.push({ role: 'title', id: node.id, value: title.length > 28 ? `${title.slice(0, 28)}…` : title, x: p.x + r + 5, y: p.y + 17, font: `${selected ? 13 : 11}px ${pal.sans}`, align: 'left', color: 'rgba(185,179,166,.95)', alpha });
    }
    return texts;
  }

  /* One node: its dot, its names, and whether its label is still wanted. */
  function nodePlan(view, w, h, near, node, pal) {
    const p = span(view, node.x, node.y);
    if (p.x < -40 || p.y < -40 || p.x > w + 40 || p.y > h + 40) return null;
    const k = view.cam.k, selected = node.id === view.selected, hover = node.id === view.hover;
    const r = nodeRadius(node, k), alpha = nodeAlpha(near, node.id);
    return {
      entry: {
        id: node.id,
        circle: { x: p.x, y: p.y, r: r + (selected ? 3 : 0), alpha, fill: node.open ? pal[node.kind] : 'rgba(13,18,32,1)', stroke: selected || hover ? '#ece6d8' : pal[node.kind], lineWidth: selected ? 2.5 : 1.2 },
        texts: textPlan(view, node, p, r, alpha, selected, hover, pal),
      },
      wanted: k >= ID_K && !view.labels.has(node.id),
    };
  }

  /* Every node on the canvas, in order, and the ids whose labels are missing. */
  function nodesPlan(view, w, h, near, pal) {
    const nodes = [], wanted = [];
    for (const node of view.graph.nodes) {
      const made = nodePlan(view, w, h, near, node, pal);
      if (!made) continue;
      if (made.wanted) wanted.push(node.id);
      nodes.push(made.entry);
    }
    return { nodes, wanted };
  }

  /* The display list, in the order the frame paints it, and the labels the
     caller should still ask for. Pure data: nothing is drawn here. */
  function plan(view, w, h, pal) {
    const near = view.selected ? neighbours(view) : null;
    const dim = near ? 0.22 : 1;
    const nodes = nodesPlan(view, w, h, near, pal);
    return {
      bubbles: bubblePlan(view, pal),
      cross: crossPlan(view, near, dim),
      edges: edgesPlan(view, w, h, near, dim),
      nodes: nodes.nodes,
      wanted: nodes.wanted,
    };
  }

  /* One disc, one line, one run of text: the whole of the drawing. */
  function disc(ctx, op) {
    ctx.globalAlpha = op.alpha ?? 1;
    ctx.beginPath(); ctx.arc(op.x, op.y, op.r, 0, Math.PI * 2);
    ctx.fillStyle = op.fill; ctx.fill();
    ctx.strokeStyle = op.stroke; ctx.lineWidth = op.lineWidth; ctx.stroke();
  }
  function line(ctx, op) {
    ctx.globalAlpha = op.alpha ?? 1;
    ctx.strokeStyle = op.stroke; ctx.lineWidth = op.lineWidth;
    ctx.beginPath(); ctx.moveTo(op.x1, op.y1); ctx.lineTo(op.x2, op.y2); ctx.stroke();
  }
  function words(ctx, op) {
    ctx.globalAlpha = op.alpha ?? 1;
    ctx.font = op.font; ctx.textAlign = op.align; ctx.fillStyle = op.color;
    ctx.fillText(op.value, op.x, op.y);
  }

  /* The list, painted: no judgement is made here. */
  function draw(ctx, list) {
    for (const bubble of list.bubbles) { disc(ctx, bubble.circle); words(ctx, bubble.text); }
    for (const cross of list.cross) line(ctx, cross);
    for (const edge of list.edges) line(ctx, edge);
    for (const node of list.nodes) {
      disc(ctx, node.circle);
      for (const text of node.texts) words(ctx, text);
    }
    ctx.globalAlpha = 1;
  }

  /* One frame: judge (plan), paint (draw), and answer the ids whose labels the
     caller should ask for. The canvas is handed in by its owner (n-88b2). */
  function frame(c, view) {
    if (!c || !view.graph) return [];
    const dpr = window.devicePixelRatio || 1, w = c.clientWidth, h = c.clientHeight;
    if (c.width !== w * dpr) { c.width = w * dpr; c.height = h * dpr; }
    const ctx = c.getContext('2d');
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    const list = plan(view, w, h, colours());
    draw(ctx, list);
    last = list;
    return list.wanted;
  }

  /* The trail, the scale, and the step it is on. */
  function crumb(c, view, t, step) {
    const node = view.selected ? find(view, view.selected) : null;
    const scope = node ? node.scope : scopeUnder(c, view);
    return `<button id="gback">${t('gBack')}</button><span id="lod" style="color:var(--paper-3)">×${view.cam.k.toFixed(2)} · ${t(step)}</span>` +
      (scope ? `<span>›</span><button data-scope="${scope}">${scope}</button>` : '') +
      (node ? `<span>›</span><span style="color:var(--paper)">${alias(view, node.id)}</span>` : '');
  }

  /* The bubble the middle of the view is inside, once the scale shows one. */
  function scopeUnder(c, view) {
    if (!c || view.cam.k <= EDGE_K || !view.graph) return null;
    const wx = (c.clientWidth / 2 - view.cam.x) / view.cam.k, wy = (c.clientHeight / 2 - view.cam.y) / view.cam.k;
    const found = view.graph.bubbles.find(bubble => Math.hypot(bubble.x - wx, bubble.y - wy) < bubble.r);
    return found ? found.scope : null;
  }

  /* The node under a canvas point, if any. */
  function pick(view, px, py) {
    let best = null, close = 1e9;
    for (const node of view.graph.nodes) {
      const p = span(view, node.x, node.y);
      const r = Math.max(6, nodeRadius(node, view.cam.k) + 4);
      const d = Math.hypot(p.x - px, p.y - py);
      if (d < r && d < close) { best = node.id; close = d; }
    }
    return best;
  }

  function pickBubble(view, px, py) {
    return view.graph.bubbles.find(bubble => {
      const p = span(view, bubble.x, bubble.y);
      return Math.hypot(p.x - px, p.y - py) < bubble.r * view.cam.k;
    });
  }

  window.GyDraw = { colours, span, frame, crumb, scopeUnder, pick, pickBubble, lastList: () => last };
})();