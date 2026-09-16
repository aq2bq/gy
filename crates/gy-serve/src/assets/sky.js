/* The sky (d-3e8f): /api/graph shrunk to the nodes that matter, drawn inside a
   canvas of the now region. A helper, not a region: the now region owns #main
   and hands the canvas in. The projection is one function, so the picture and
   the hit test agree; the hit test is pure, and the click stays in events.js. */
(function () {
  const RING = {
    '#f28ba0': 'rgba(242,139,160,.20)',
    '#e9a63a': 'rgba(233,166,58,.20)',
    '#5fd3c7': 'rgba(95,211,199,.20)',
  };
  const REACH = 400; /* how far a click looks, in pixels */

  /* The three sets the sky lights, by column colour. */
  function lit(now) {
    return {
      '#f28ba0': new Set(now.waiting.map(item => item.row.id)),
      '#e9a63a': new Set(now.ready.map(item => item.row.id)),
      '#5fd3c7': new Set(now.resume.in_progress.map(item => item.id)),
    };
  }

  /* Where every node sits in a canvas of w by h. Pure: the fit, the scale, and
     the origin come only from the map and the two numbers. */
  function geometry(map, sets, w, h) {
    const colour = id => Object.keys(sets).find(key => sets[key].has(id)) || null;
    const litNodes = map.nodes.filter(node => colour(node.id));
    const fit = litNodes.length ? litNodes : map.nodes;
    let minx = 1e9, maxx = -1e9, miny = 1e9, maxy = -1e9;
    for (const node of fit) {
      minx = Math.min(minx, node.x); maxx = Math.max(maxx, node.x);
      miny = Math.min(miny, node.y); maxy = Math.max(maxy, node.y);
    }
    const pad = 40;
    const k = Math.min((w - pad * 2) / Math.max(1, maxx - minx), (h - pad * 2) / Math.max(1, maxy - miny));
    const ox = w / 2 - ((minx + maxx) / 2) * k;
    const oy = h / 2 - ((miny + maxy) / 2) * k;
    const points = new Map(map.nodes.map(node => [node.id, [node.x * k + ox, node.y * k + oy]]));
    return { colour, points };
  }

  /* The laid-out size of the canvas, in the pixels the fit uses. */
  function size(canvas) {
    const rect = canvas.getBoundingClientRect();
    return [Math.round(rect.width) || canvas.clientWidth, Math.round(rect.height) || canvas.clientHeight];
  }

  /* The edges, faint, under the dots. */
  function edges(ctx, map, g) {
    ctx.strokeStyle = 'rgba(185,179,166,.10)';
    ctx.lineWidth = 0.6;
    for (const [a, b] of map.edges) {
      const A = g.points.get(a), B = g.points.get(b);
      if (!A || !B) continue;
      ctx.beginPath(); ctx.moveTo(A[0], A[1]); ctx.lineTo(B[0], B[1]); ctx.stroke();
    }
  }

  /* The lit nodes as a ring and a dot; the rest as a faint speck. */
  function dots(ctx, map, g) {
    for (const node of map.nodes) {
      const [px, py] = g.points.get(node.id);
      const col = g.colour(node.id);
      if (col) {
        ctx.beginPath(); ctx.arc(px, py, 7, 0, Math.PI * 2); ctx.fillStyle = RING[col]; ctx.fill();
        ctx.beginPath(); ctx.arc(px, py, 3, 0, Math.PI * 2); ctx.fillStyle = col; ctx.fill();
      } else {
        ctx.beginPath(); ctx.arc(px, py, 1.4, 0, Math.PI * 2); ctx.fillStyle = 'rgba(185,179,166,.28)'; ctx.fill();
      }
    }
  }

  /* The whole picture: the edges, then the dots. */
  function draw(canvas, state) {
    if (!canvas || !state.map || !state.now) return;
    const [w, h] = size(canvas);
    const dpr = window.devicePixelRatio || 1;
    canvas.width = w * dpr;
    canvas.height = h * dpr;
    const ctx = canvas.getContext('2d');
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);
    const g = geometry(state.map, lit(state.now), w, h);
    edges(ctx, state.map, g);
    dots(ctx, state.map, g);
  }

  /* The lit node nearest a canvas point, or null. Pure: the same geometry the
     picture used, so a click lands on what it looks like it lands on. */
  function hit(state, canvas, clientX, clientY) {
    if (!canvas || !state.map || !state.now) return null;
    const g = geometry(state.map, lit(state.now), ...size(canvas));
    const rect = canvas.getBoundingClientRect();
    const px = clientX - rect.left;
    const py = clientY - rect.top;
    let best = null;
    let close = REACH;
    for (const [id, point] of g.points) {
      if (!g.colour(id)) continue;
      const distance = Math.hypot(point[0] - px, point[1] - py);
      if (distance < close) { close = distance; best = id; }
    }
    return best;
  }

  window.GySky = { draw, hit };
})();