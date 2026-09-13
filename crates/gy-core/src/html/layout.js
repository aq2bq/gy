// ---------- force-directed layout for the individual mode (H21) ----------
function computeForce(ids) {
  const n = ids.length;
  const W = 1400, H = 900;
  const pos = {};
  ids.forEach((id, i) => {
    const angle = 2 * Math.PI * i / Math.max(1, n);
    const r = Math.min(W, H) * 0.35;
    pos[id] = {x: W/2 + Math.cos(angle) * r, y: H/2 + Math.sin(angle) * r, vx: 0, vy: 0};
  });
  const idSet = new Set(ids);
  const spring = [];
  EDGES.forEach(e => {
    if (idSet.has(e.source) && idSet.has(e.target)) spring.push([e.source, e.target]);
  });
  for (let iter = 0; iter < 400; iter++) {
    for (let i = 0; i < n; i++) {
      const a = pos[ids[i]];
      for (let j = i + 1; j < n; j++) {
        const b = pos[ids[j]];
        let dx = a.x - b.x, dy = a.y - b.y;
        let d = Math.max(1, Math.hypot(dx, dy));
        const f = 160000 / (d * d);
        dx /= d; dy /= d;
        a.vx += dx * f; a.vy += dy * f;
        b.vx -= dx * f; b.vy -= dy * f;
      }
    }
    spring.forEach(([s, t]) => {
      const a = pos[s], b = pos[t];
      let dx = a.x - b.x, dy = a.y - b.y;
      const d = Math.max(1, Math.hypot(dx, dy));
      const f = 0.03 * (d - 110);
      dx /= d; dy /= d;
      a.vx -= dx * f; a.vy -= dy * f;
      b.vx += dx * f; b.vy += dy * f;
    });
    ids.forEach(id => {
      const p = pos[id];
      p.vx += (W/2 - p.x) * 0.001;
      p.vy += (H/2 - p.y) * 0.001;
      const damp = 0.85;
      p.x += p.vx * damp; p.y += p.vy * damp;
      p.vx = 0; p.vy = 0;
    });
  }
  // Normalize the layout to a size proportional to the node count, so that at
  // fit scale ~1 the labels are readable (H28). Nodes sit ~70 units apart.
  let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
  ids.forEach(id => { const p = pos[id]; minX = Math.min(minX,p.x); minY = Math.min(minY,p.y); maxX = Math.max(maxX,p.x); maxY = Math.max(maxY,p.y); });
  const spanX = maxX - minX, spanY = maxY - minY;
  const target = Math.max(320, Math.sqrt(n) * 70);
  const k = target / Math.max(1, Math.max(spanX, spanY));
  const cx = (minX + maxX) / 2, cy = (minY + maxY) / 2;
  ids.forEach(id => { const p = pos[id]; p.x = (p.x - cx) * k; p.y = (p.y - cy) * k; });
  return pos;
}
function ensureForce(ids) {
  const key = [...ids].sort().join('|');
  if (forceKey !== key) { forceLayout = computeForce(ids); forceKey = key; }
}

// ---------- layout: global grid by (scope, type) ----------
function initLayout() {
  const order = [];
  const seen = new Set();
  NODES.forEach(n => { const g = n.scope + '\u0000' + n.type; if (!seen.has(g)) { seen.add(g); order.push(g); } });
  order.sort();
  const cols = Math.ceil(Math.sqrt(order.length));
  const colW = 150 * Math.sqrt(NODES.length / Math.max(1, order.length));
  for (const k of Object.keys(groupPos)) delete groupPos[k];
  order.forEach((g, gi) => {
    const [scope, type] = g.split('\u0000');
    const members = NODES.filter(n => n.scope === scope && n.type === type).sort((a,b) => parseInt(a.id.replace(/\D/g,''),10) - parseInt(b.id.replace(/\D/g,''),10));
    const perRow = Math.max(6, Math.ceil(Math.sqrt(members.length * 2)));
    const bx = (gi % cols) * (colW + 90);
    const by = Math.floor(gi / cols) * (120 + Math.ceil(members.length / perRow) * 36);
    groupPos[g] = {bx, by, type, members: members.map(m => m.id)};
    members.forEach((n, i) => {
      positions[n.id] = { x: bx + (i % perRow) * 26, y: by + Math.floor(i / perRow) * 34 };
    });
  });
}

// genealogy layered layout
function buildGenealogy() {
  const decisionIds = NODES.filter(n => n.type === 'decision').map(n => n.id);
  const dSet = new Set(decisionIds);
  const geneEdges = EDGES.filter(e => e.label === 'narrows' || e.label === 'supersedes' || e.label === 'completes' || e.label === 'widens');
  const depends = {};  // a -> nodes that a modifies/replaces (edge target)
  geneEdges.forEach(e => { (depends[e.source] = depends[e.source] || []).push(e.target); });
  const depth = {};
  function computeDepth(id, seen) {
    if (depth[id] !== undefined) return depth[id];
    if (seen.has(id)) return 0;
    const deps = (depends[id] || []).filter(t => dSet.has(t));
    let d = 0;
    deps.forEach(t => { d = Math.max(d, 1 + computeDepth(t, new Set(seen).add(id))); });
    depth[id] = d;
    return d;
  }
  decisionIds.forEach(id => computeDepth(id, new Set()));
  const layers = {};
  decisionIds.forEach(id => { const d = depth[id] === undefined ? 0 : depth[id]; (layers[d] = layers[d] || []).push(id); });
  let maxW = 0;
  Object.keys(layers).forEach(k => maxW = Math.max(maxW, layers[k].length));
  genealogyLayout = {};
  const spacing = 90, layerGap = 190;
  Object.keys(layers).sort((a,b)=>a-b).forEach((k, li) => {
    const ids = layers[k].sort((a,b)=>a.localeCompare(b));
    ids.forEach((id, j) => {
      genealogyLayout[id] = { x: li * layerGap, y: (j - (ids.length-1)/2) * 64 };
    });
  });
}

