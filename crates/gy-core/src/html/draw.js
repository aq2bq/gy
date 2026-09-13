// ---------- draw dispatch ----------
// Compute overview cluster centers so the grid matches the viewport aspect.
function overviewGrid(ids) {
  const cluster = {};
  ids.forEach(id => { const n = byId[id]; const g = n.scope + '\u0000' + n.type; (cluster[g] = cluster[g] || []).push(id); });
  const gs = Object.keys(cluster);
  const cells = {};
  if (!gs.length) return {cluster, cells};
  const cellW = 320, cellH = 150;
  const vp = svg.getBoundingClientRect();
  const aspect = Math.max(0.2, Math.min(5, (vp.width || 800) / (vp.height || 600)));
  const n = gs.length;
  // Prefer at least two columns so inter-cluster edges gain horizontal travel and
  // their labels do not stack on a single vertical line.
  let cols = n, rows = 1, best = Infinity;
  for (let c = (n > 1 ? 2 : 1); c <= n; c++) {
    const r = Math.ceil(n / c);
    const score = Math.abs((c * cellW) / (r * cellH) - aspect);
    if (score < best) { best = score; cols = c; rows = r; }
  }
  const gridW = cols * cellW, gridH = rows * cellH;
  gs.forEach((g, i) => {
    const c = i % cols, r = Math.floor(i / cols);
    cells[g] = {bx: c * cellW + cellW / 2 - gridW / 2, by: r * cellH + cellH / 2 - gridH / 2};
  });
  return {cluster, cells};
}

function dotPath(sx, sy, tx, ty) {
  const dx = tx-sx, dy = ty-sy, d = Math.max(1, Math.hypot(dx,dy));
  return {x1: sx, y1: sy, x2: tx - dx/d*16, y2: ty - dy/d*16};
}

// Refit automatically whenever the visible set changes (overview <-> neighborhood,
// search/filter, genealogy entry), so the new content is not left outside the viewport.
let lastFitKey = '';
let lastViewport = null;
function draw() {
  const ids = visibleNodes();
  renderNavigation(ids);
  const viewport = svg.getBoundingClientRect();
  const focus = currentFocus();
  const key = JSON.stringify([genealogyMode, focus.id, focus.radius, [...ids].sort()]);
  if (key !== lastFitKey) {
    lastFitKey = key;
    fitTransform();
  } else if (lastViewport && (viewport.width !== lastViewport.width || viewport.height !== lastViewport.height)) {
    if (viewSource === 'fit') fitTransform();
    else {
      // Preserve the world point at the viewport center and the user's scale.
      translate.x += (viewport.width - lastViewport.width) / 2;
      translate.y += (viewport.height - lastViewport.height) / 2;
      applyTransform();
    }
  }
  lastViewport = {width: viewport.width, height: viewport.height};
  const showingAll = ids.length <= MODE_THRESHOLD || genealogyMode;
  if (!genealogyMode && showingAll && ids.length > 0) ensureForce(ids);
  const inPositions = genealogyMode ? genealogyLayout : (showingAll ? forceLayout : positions);

  // clear
  root.innerHTML = '';
  document.getElementById('lodInfo').textContent =
    showingAll ? 'zoom level: ' + lod + ' (scale ' + scale.toFixed(2) + ') '
    : 'overview · zoom level: ' + lod + ' (scale ' + scale.toFixed(2) + ')';

  const culling = document.getElementById('culling');
  const setMeta = (drawn, visCount, extra) => {
    document.getElementById('graphCount').textContent = drawn;
    document.getElementById('graphVisible').textContent = visCount;
    if (extra) { culling.style.display = 'block'; culling.textContent = extra; }
    else { culling.style.display = 'none'; culling.textContent = ''; }
  };

  if (genealogyMode) {
    drawEdgesLayer(inPositions, ids.filter(id=>inPositions[id]), visibleEdges(ids).filter(e => (e.label === 'narrows' || e.label === 'supersedes' || e.label === 'completes' || e.label === 'widens') && inPositions[e.source] && inPositions[e.target]));
    const n = drawNodeLayer(inPositions, ids);
    const off = offscreenCount(inPositions, ids);
    setMeta(n - off, n, off > 0 ? ('Showing ' + (n - off) + ' of ' + n + ' nodes; ' + off + ' off-screen at readable zoom. Zoom out or pan to reach them.') : '');
    return;
  }

  if (!showingAll) {
    // Overview: clusters + aggregated edges, no individual nodes.  ------
    const {cluster, cells} = overviewGrid(ids);
    const cg = Object.keys(cluster);
    // Aggregated edges between clusters (H19)
    const pair = {};
    EDGES.forEach(e => {
      if (!ids.includes(e.source) || !ids.includes(e.target)) return;
      const gs = byId[e.source].scope + '\u0000' + byId[e.source].type;
      const gt = byId[e.target].scope + '\u0000' + byId[e.target].type;
      if (gs === gt) return;                 // kept as internal count (H26)
      const k = gs + '|' + gt;
      (pair[k] = pair[k] || []).push(e.label);
    });
    const edgeLayer = document.createElementNS(NS, 'g');
    const pairKeys = Object.keys(pair);
    // Fan inter-cluster edges out with quadratic curves so that edges sharing a
    // lane separate and every label can be traced back to its two clusters (H19).
    pairKeys.forEach((k, i) => {
      const labels = pair[k];
      const [gs, gt] = k.split('|');
      const a = cells[gs] || {bx:0, by:0};
      const b = cells[gt] || {bx:0, by:0};
      const sx = a.bx, sy = a.by, tx = b.bx, ty = b.by;
      const dx = tx - sx, dy = ty - sy, len = Math.max(1, Math.hypot(dx, dy));
      // perpendicular unit vector; alternate sides by pair index
      const ux = -dy / len, uy = dx / len;
      const side = (i % 2 === 0 ? 1 : -1);
      const dist = 90 + Math.floor(i / 2) * 28;
      const mx = (sx + tx) / 2 + ux * dist * side;
      const my = (sy + ty) / 2 + uy * dist * side;
      const endX = tx - dx / len * 16, endY = ty - dy / len * 16;
      const path = document.createElementNS(NS, 'path');
      path.setAttribute('d', 'M ' + sx + ' ' + sy + ' Q ' + mx + ' ' + my + ' ' + endX + ' ' + endY);
      path.setAttribute('fill', 'none');
      path.setAttribute('stroke', '#888');
      path.setAttribute('stroke-width', '1.4');
      path.setAttribute('marker-end', 'url(#arr-_agg)');
      path.setAttribute('opacity', '0.75');
      path.setAttribute('data-label', [...new Set(labels)].join(','));
      path.setAttribute('data-s', gs);
      path.setAttribute('data-t', gt);
      edgeLayer.appendChild(path);
      const t = document.createElementNS(NS, 'text');
      t.setAttribute('x', mx + ux * 14 * side); t.setAttribute('y', my + uy * 14 * side - 4);
      t.setAttribute('text-anchor', 'middle'); t.setAttribute('class', 'elabel');
      t.setAttribute('data-count', labels.length);
      t.setAttribute('data-s', gs);
      t.setAttribute('data-t', gt);
      t.textContent = labels.length > 1 ? labels.length + '' : labels[0];
      edgeLayer.appendChild(t);
    });
    root.appendChild(edgeLayer);

    // Internal closed edges per cluster (H26)
    const internal = {};
    EDGES.forEach(e => {
      if (!ids.includes(e.source) || !ids.includes(e.target)) return;
      const g = byId[e.source].scope + '\u0000' + byId[e.source].type;
      if (g !== byId[e.target].scope + '\u0000' + byId[e.target].type) return;
      (internal[g] = internal[g] || []).push(e.label);
    });

    cg.forEach(g => {
      const [scope, type] = g.split('\u0000');
      const gp = cells[g] || {bx:0, by:0};
      const members = cluster[g];
      const w = Math.min(280, Math.max(120, 40 + members.length * 1.4)), h = 34;
      const rect = document.createElementNS(NS, 'rect');
      rect.setAttribute('x', gp.bx - w/2); rect.setAttribute('y', gp.by - h/2);
      rect.setAttribute('width', w); rect.setAttribute('height', h);
      rect.setAttribute('rx', '8');
      rect.setAttribute('fill', KIND_COLORS[type]); rect.setAttribute('opacity', '0.6');
      rect.setAttribute('class', 'node');
      rect.addEventListener('click', () => openClusterPanel(g, members));
      root.appendChild(rect);
      const t1 = document.createElementNS(NS, 'text');
      t1.setAttribute('x', gp.bx); t1.setAttribute('y', gp.by - 1);
      t1.setAttribute('text-anchor', 'middle'); t1.setAttribute('class', 'nlabel'); t1.style.fill = '#111';
      t1.textContent = scope + ' / ' + type + ' · ' + members.length;
      root.appendChild(t1);
      const t2 = document.createElementNS(NS, 'text');
      t2.setAttribute('x', gp.bx); t2.setAttribute('y', gp.by + 13);
      t2.setAttribute('text-anchor', 'middle'); t2.setAttribute('class', 'elabel'); t2.style.fill = '#333';
      t2.textContent = internal[g] ? ('internal ' + internal[g].length) : '';
      root.appendChild(t2);
    });
    setMeta(cg.length, ids.length + ' nodes', cg.length + ' clusters · ' + ids.length + ' nodes available');
    return;
  }

  // Individual mode / filtered small set: force layout (H21)
  drawEdgesLayer(inPositions, ids, visibleEdges(ids));
  const n = drawNodeLayer(inPositions, ids);
  // H28 + H18: at the readable fit scale some nodes may fall outside the viewport.
  const off = offscreenCount(inPositions, ids);
  const noted = off > 0 ? ('Showing ' + (n - off) + ' of ' + n + ' nodes; ' + off + ' off-screen at readable zoom. Zoom out or pan to reach them.') : '';
  setMeta(n - off, n, noted);
}

function offscreenCount(layout, ids) {
  const rect = svg.getBoundingClientRect();
  let off = 0;
  ids.forEach(id => {
    const p = layout[id];
    if (!p) return;
    const sx = p.x * scale + translate.x;
    const sy = p.y * scale + translate.y;
    if (sx < -20 || sx > rect.width + 20 || sy < -20 || sy > rect.height + 20) off++;
  });
  return off;
}

function drawEdgesLayer(inPositions, ids, drawEdges) {
  const edgeLayer = document.createElementNS(NS, 'g');
  const wantLabels = !genealogyMode && ids.length <= EDGE_LABEL_MAX;
  drawEdges.forEach((e, idx) => {
    const a = inPositions[e.source], b = inPositions[e.target];
    if (!a || !b) return;
    const line = document.createElementNS(NS, 'line');
    const p = dotPath(a.x, a.y, b.x, b.y);
    line.setAttribute('x1', p.x1); line.setAttribute('y1', p.y1);
    line.setAttribute('x2', p.x2); line.setAttribute('y2', p.y2);
    line.setAttribute('stroke', edgeColor(e.label));
    line.setAttribute('stroke-width', '1.2');
    line.setAttribute('marker-end', 'url(#arr-' + e.label.replace(/\W/g, '_') + ')');
    line.setAttribute('opacity', '0.5');
    line.setAttribute('data-lbl', e.label);
    line.setAttribute('data-idx', idx);
    edgeLayer.appendChild(line);
    if (wantLabels) {
      const t = document.createElementNS(NS, 'text');
      t.setAttribute('x', (p.x1+p.x2)/2); t.setAttribute('y', (p.y1+p.y2)/2 - 3);
      t.setAttribute('text-anchor', 'middle'); t.setAttribute('class', 'elabel');
      t.textContent = e.label;
      edgeLayer.appendChild(t);
    }
  });
  root.appendChild(edgeLayer);
}

function drawNodeLayer(inPositions, ids) {
  const nodeLayer = document.createElementNS(NS, 'g');
  const placed = [];
  let drawn = 0;
  ids.forEach(id => {
    const n = byId[id];
    const p = inPositions[id];
    if (!p) return;
    const isSelected = selected === id;
    const isHit = searchHits && searchHits.has(id);
    const g = document.createElementNS(NS, 'g');
    g.setAttribute('transform', 'translate(' + p.x + ',' + p.y + ')');
    g.setAttribute('class', 'node');
    if (isSelected) {
      const ring = document.createElementNS(NS, 'circle');
      ring.setAttribute('r', '13'); ring.setAttribute('class', 'ring');
      ring.setAttribute('fill', 'none'); ring.setAttribute('stroke', 'var(--hl)'); ring.setAttribute('stroke-width', '3');
      g.appendChild(ring);
    }
    if (isHit) {
      const hit = document.createElementNS(NS, 'circle');
      hit.setAttribute('r', '15'); hit.setAttribute('fill', 'none'); hit.setAttribute('stroke', '#ffb000'); hit.setAttribute('stroke-width', '2');
      g.appendChild(hit);
    }
    const r = lod === 'near' ? 11 : 8;
    const path = document.createElementNS(NS, 'path');
    path.setAttribute('d', shapeOf(n.type));
    path.setAttribute('fill', isSelected ? '#ffd766' : KIND_COLORS[n.type]);
    path.setAttribute('stroke', '#222'); path.setAttribute('stroke-width', '0.8');
    path.setAttribute('transform', 'scale(' + (r/8) + ')');
    g.appendChild(path);
    if (n.type === 'requirement' && n.status === 'complete') {
      const ck = document.createElementNS(NS, 'text');
      ck.setAttribute('x', 0); ck.setAttribute('y', 3); ck.setAttribute('text-anchor', 'middle');
      ck.style.fontSize = '9px'; ck.style.fill = '#fff'; ck.textContent = '✓';
      g.appendChild(ck);
    }
    if (n.type === 'decision' && n.superseded_by && n.superseded_by.length) {
      const x = document.createElementNS(NS, 'text');
      x.setAttribute('x', 0); x.setAttribute('y', 3); x.setAttribute('text-anchor', 'middle');
      x.style.fontSize = '9px'; x.style.fill = '#fff'; x.textContent = '×';
      x.setAttribute('transform', 'scale(0.8)');
      g.appendChild(x);
    }
    // label: fit available width, measured via a live probe, skip on overlap (H22)
    const probe = document.createElementNS(NS, 'text');
    probe.setAttribute('text-anchor', 'middle');
    probe.setAttribute('class', 'nlabel');
    svg.appendChild(probe);               // must be connected to measure
    const full = lod === 'near'
      ? (n.id + ' · ' + n.title + (n.status ? ' · ' + n.status : ''))
      : n.id;
    probe.textContent = full;
    let tw = probe.getComputedTextLength() || full.length * 6;
    const budget = levelW();
    while (tw > budget && probe.textContent.length > 4) {
      probe.textContent = probe.textContent.slice(0, probe.textContent.length - 2) + '…';
      tw = probe.getComputedTextLength() || probe.textContent.length * 6;
    }
    const finalText = probe.textContent;
    svg.removeChild(probe);
    // overlap: BBox collision against placed labels
    const pad = 6;
    const bb = {x: p.x - tw/2, y: p.y + r + 3, w: tw + pad, h: 13 + pad};
    const collides = placed.some(q => !(bb.x + bb.w < q.x || q.x + q.w < bb.x || bb.y + bb.h < q.y || q.y + q.h < bb.y));
    if (!collides && tw <= budget) {
      const t = document.createElementNS(NS, 'text');
      t.setAttribute('x', 0); t.setAttribute('y', r + 13);
      t.setAttribute('text-anchor', 'middle'); t.setAttribute('class', 'nlabel');
      t.textContent = finalText;
      g.appendChild(t);
      placed.push(bb);
    }
    g.dataset.nodeId = id;
    g.setAttribute('tabindex', '0');
    g.setAttribute('role', 'button');
    g.setAttribute('aria-label', 'Focus ' + id + ' · ' + n.title);
    g.addEventListener('click', (e) => { e.stopPropagation(); if (!dragMoved) startHop(id); });
    g.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); startHop(id); }
    });
    nodeLayer.appendChild(g);
    drawn++;
  });
  root.appendChild(nodeLayer);
  return drawn;
}

function levelW() { return scale >= 1.1 ? MAX_LABEL_W : MAX_LABEL_W * 0.6; }

