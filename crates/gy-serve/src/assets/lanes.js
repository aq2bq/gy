/* The cross screen's placement (a part; d-c302, d-6d90): rows are scopes and
   the width is time. Nothing here is a force: an item's place comes from its
   time and the row it lands in, so a node with five hundred edges cannot make
   a coordinate run away (ac-afba). No DOM, no fetch. */
(function () {
  const DAY = 86400;
  const L = 132;
  const R = 22;
  const LANES = 24;
  const LANE_H = 9;
  const MAX_ROW = 320;
  const TOP = 34;

  const radius = degree => 3 + Math.min(6, Math.sqrt(Math.max(0, degree || 0)) * 1.4);
  const finite = value => (Number.isFinite(value) ? value : 0);

  /* A time within the window, clamped; a missing or broken one is the start. */
  function clamp(time, from, to) {
    if (!Number.isFinite(time)) return from;
    return Math.min(to, Math.max(from, time));
  }

  /* The asked period, else what the set spans (one day either side). */
  function timeWindow(nodes, input) {
    const times = nodes.map(node => node.created).filter(value => Number.isFinite(value));
    const low = times.length ? Math.min(...times) : 0;
    const high = times.length ? Math.max(...times) : low + DAY;
    const from = Number.isFinite(input && input.from) ? input.from : low - DAY;
    const to = Number.isFinite(input && input.to) ? input.to : high + DAY;
    return { from, to, span: Math.max(1, to - from) };
  }

  function timeScale(window, width) {
    return time => L + ((clamp(time, window.from, window.to) - window.from) / window.span) * (width - L - R);
  }

  /* The degrees give the dots their size; an edge counts at both ends. */
  function degrees(links) {
    const found = new Map();
    for (const link of links) {
      found.set(link.from, (found.get(link.from) || 0) + 1);
      found.set(link.to, (found.get(link.to) || 0) + 1);
    }
    return found;
  }

  /* The rows: the scopes with a node, in the asked order, then any other. */
  function scopeOrder(nodes, asked) {
    const seen = new Set();
    const scopes = [];
    for (const name of asked) {
      if (seen.has(name)) continue;
      if (!nodes.some(node => node.scope === name)) continue;
      seen.add(name);
      scopes.push(name);
    }
    for (const node of nodes) {
      if (seen.has(node.scope)) continue;
      seen.add(node.scope);
      scopes.push(node.scope);
    }
    return scopes;
  }

  function rowItems(nodes, scope, time, degree) {
    return nodes
      .filter(node => node.scope === scope)
      .map(node => ({ node, x: finite(time(node.created)), r: radius(node.degree !== undefined ? node.degree : degree.get(node.id)), lane: 0 }))
      .sort((a, b) => a.x - b.x || String(a.node.id).localeCompare(String(b.node.id)));
  }

  /* A lane holds items that do not overlap in x. A full row wraps to a lane
     near the middle, so the offset stays bounded. */
  function pickLanes(items, width) {
    const lastX = new Array(LANES).fill(-Infinity);
    let used = 0;
    for (const item of items) {
      let lane = 0;
      while (lane < LANES && item.x - lastX[lane] < item.r + 10) lane++;
      if (lane >= LANES) lane = Math.min(LANES - 1, Math.floor((item.x / Math.max(1, width)) * LANES));
      item.lane = lane;
      lastX[lane] = item.x;
      used = Math.max(used, lane + 1);
    }
    return used || 1;
  }

  /* One row's band and points, from the lanes its items took. */
  function placeRow(items, width, top, core) {
    const used = pickLanes(items, width);
    const wanted = used * LANE_H + 24;
    const height = Math.max(54, Math.min(MAX_ROW, wanted));
    const squeeze = wanted > MAX_ROW ? (MAX_ROW - 24) / (used * LANE_H) : 1;
    const middle = top + height / 2;
    const points = items.map(item => ({
      id: item.node.id,
      node: item.node,
      x: finite(item.x),
      y: finite(middle + (item.lane - (used - 1) / 2) * LANE_H * squeeze),
      r: finite(item.r),
      core: !!core[item.node.id],
      lane: item.lane,
    }));
    return { height, points };
  }

  function linkEdges(links, byPoint) {
    const edges = [];
    for (const link of links) {
      const a = byPoint.get(link.from);
      const b = byPoint.get(link.to);
      if (!a || !b) continue;
      edges.push({
        from: link.from,
        to: link.to,
        relation: link.relation,
        cross: a.node.scope !== b.node.scope,
        ghost: !a.core || !b.core,
        a,
        b,
      });
    }
    return edges;
  }

  /* Place the set: points by id, edges with both ends, and the rows' bands. The
     `core` map names the nodes that were asked for; the rest are one step out
     and are drawn faintly. */
  function layout(input) {
    const nodes = (input && input.nodes) || [];
    const links = (input && input.edges) || [];
    const width = Math.max(320, finite((input && input.width) || 900));
    const core = (input && input.core) || {};
    const asked = (input && input.scopes) || [];
    const window = timeWindow(nodes, input);
    const time = timeScale(window, width);
    const degree = degrees(links);
    const byPoint = new Map();
    const rows = [];
    let y = TOP;
    for (const scope of scopeOrder(nodes, asked)) {
      const items = rowItems(nodes, scope, time, degree);
      const placed = placeRow(items, width, y, core);
      const mine = nodes.filter(node => node.scope === scope && core[node.id]);
      rows.push({
        scope,
        y,
        h: placed.height,
        count: mine.length,
        open: mine.filter(node => isOpen(node.state)).length,
      });
      for (const point of placed.points) byPoint.set(point.id, point);
      y += placed.height;
    }
    return {
      width,
      height: Math.max(120, y + 10),
      from: window.from,
      to: window.to,
      xOf: time,
      rows,
      points: [...byPoint.values()],
      edges: linkEdges(links, byPoint),
    };
  }

  /* The states the cross screen counts as open (the server's words). */
  const OPEN = new Set(['open', 'filed', 'approved', 'unmet', 'live', 'in progress', '進行中', '起票', '確定', '未達', '有効']);
  const isOpen = value => OPEN.has(String(value ?? '').toLowerCase());

  window.GyLanes = { layout, radius, isOpen };
})();
