/* The canvas half of the graph page (n-4cd8): one frame per call, in the
   prototype's order. The state lives in graph.js and arrives as `view`. */
(function () {
  /* The scales where the edges, the ids, and the titles appear. */
  const EDGE_K = 0.9, ID_K = 1.6, TITLE_K = 3.2;
  let palette = null;

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

  /* One frame: bubbles, the edges across and inside, the nodes, the names.
     Answers the ids whose labels the caller should ask for. */
  function frame(view) {
    const c = document.getElementById('g');
    if (!c || !view.graph) return [];
    const dpr = window.devicePixelRatio || 1, w = c.clientWidth, h = c.clientHeight;
    if (c.width !== w * dpr) { c.width = w * dpr; c.height = h * dpr; }
    const ctx = c.getContext('2d'), pal = colours(), cam = view.cam;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    const near = view.selected ? neighbours(view) : null, dim = near ? 0.22 : 1;

    for (const bubble of view.graph.bubbles) {
      const p = span(view, bubble.x, bubble.y), r = bubble.r * cam.k;
      ctx.beginPath(); ctx.arc(p.x, p.y, r, 0, Math.PI * 2);
      ctx.fillStyle = 'rgba(28,36,56,.35)'; ctx.fill();
      ctx.strokeStyle = 'rgba(42,52,80,.9)'; ctx.lineWidth = 1; ctx.stroke();
      const size = Math.max(11, Math.min(22, 14 * cam.k));
      ctx.font = `${size}px ${pal.mono}`; ctx.textAlign = 'center';
      ctx.fillStyle = cam.k < EDGE_K ? 'rgba(236,230,216,.85)' : 'rgba(126,122,112,.9)';
      ctx.fillText(bubble.scope + (cam.k < EDGE_K ? `  ${bubble.count}` : ''), p.x, p.y - r - 8);
    }

    ctx.lineWidth = Math.max(0.5, 0.8 * cam.k);
    for (const [a, b] of view.graph.cross) {
      const from = find(view, a), to = find(view, b);
      if (!from || !to) continue;
      const hi = near && (near.has(a) || near.has(b));
      ctx.strokeStyle = hi ? 'rgba(236,230,216,.9)' : `rgba(95,211,199,${0.12 * dim})`;
      const A = span(view, from.x, from.y), B = span(view, to.x, to.y);
      ctx.beginPath(); ctx.moveTo(A.x, A.y); ctx.lineTo(B.x, B.y); ctx.stroke();
    }

    for (const bubble of view.graph.bubbles) {
      if (bubble.r * cam.k < 150) continue;
      for (const [a, b] of view.graph.edges) {
        const from = find(view, a), to = find(view, b);
        if (!from || !to || from.scope !== bubble.scope) continue;
        const A = span(view, from.x, from.y), B = span(view, to.x, to.y);
        if ((A.x < 0 && B.x < 0) || (A.y < 0 && B.y < 0) || (A.x > w && B.x > w) || (A.y > h && B.y > h)) continue;
        const hi = near && near.has(a) && near.has(b) && (a === view.selected || b === view.selected);
        ctx.strokeStyle = hi ? 'rgba(236,230,216,.95)' : `rgba(185,179,166,${0.35 * dim})`;
        ctx.lineWidth = hi ? Math.max(1.2, 1.4 * cam.k) : Math.max(0.4, 0.7 * cam.k);
        ctx.beginPath(); ctx.moveTo(A.x, A.y); ctx.lineTo(B.x, B.y); ctx.stroke();
      }
    }

    const wanted = [];
    for (const node of view.graph.nodes) {
      const p = span(view, node.x, node.y);
      if (p.x < -40 || p.y < -40 || p.x > w + 40 || p.y > h + 40) continue;
      if (cam.k >= ID_K && !view.labels.has(node.id)) wanted.push(node.id);
      const r = Math.max(1.6, (2.2 + Math.sqrt(node.degree) * 1.3) * Math.min(cam.k, 4));
      const isSelected = node.id === view.selected, isHover = node.id === view.hover;
      ctx.globalAlpha = !near || near.has(node.id) ? 1 : 0.22;
      ctx.beginPath(); ctx.arc(p.x, p.y, r + (isSelected ? 3 : 0), 0, Math.PI * 2);
      /* A node that is not open is hollow: the outline alone. */
      ctx.fillStyle = node.open ? pal[node.kind] : 'rgba(13,18,32,1)'; ctx.fill();
      ctx.lineWidth = isSelected ? 2.5 : 1.2;
      ctx.strokeStyle = isSelected || isHover ? '#ece6d8' : pal[node.kind]; ctx.stroke();
      if (cam.k < ID_K && !isSelected && !isHover) { ctx.globalAlpha = 1; continue; }
      ctx.font = `${isSelected ? 12 : 10}px ${pal.mono}`; ctx.textAlign = 'left';
      ctx.fillStyle = 'rgba(236,230,216,.9)';
      ctx.fillText(alias(view, node.id), p.x + r + 5, p.y + 3.5);
      if (cam.k >= TITLE_K || isSelected || isHover) {
        const title = (view.labels.get(node.id) || {}).title || '';
        ctx.font = `${isSelected ? 13 : 11}px ${pal.sans}`; ctx.fillStyle = 'rgba(185,179,166,.95)';
        ctx.fillText(title.length > 28 ? `${title.slice(0, 28)}…` : title, p.x + r + 5, p.y + 17);
      }
      ctx.globalAlpha = 1;
    }
    return wanted;
  }

  /* The trail, the scale, and the step it is on. */
  function crumb(view, t, step) {
    const node = view.selected ? find(view, view.selected) : null;
    const scope = node ? node.scope : scopeUnder(view);
    return `<button id="gback">${t('gBack')}</button><span id="lod" style="color:var(--paper-3)">×${view.cam.k.toFixed(2)} · ${t(step)}</span>` +
      (scope ? `<span>›</span><button data-scope="${scope}">${scope}</button>` : '') +
      (node ? `<span>›</span><span style="color:var(--paper)">${alias(view, node.id)}</span>` : '');
  }

  /* The bubble the middle of the view is inside, once the scale shows one. */
  function scopeUnder(view) {
    const c = document.getElementById('g');
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
      const r = Math.max(6, (2.2 + Math.sqrt(node.degree) * 1.3) * Math.min(view.cam.k, 4) + 4);
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

  window.GyDraw = { colours, span, frame, crumb, scopeUnder, pick, pickBubble };
})();
