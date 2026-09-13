function setVisible(n) {
  if (typeState[n.type] !== true) return false;
  if (scopeSel.value && n.scope !== scopeSel.value) return false;
  if (n.type === 'requirement') {
    if (stateSel.value && (n.state || '') !== stateSel.value) return false;
  }
  if (n.type === 'question') {
    const qs = document.getElementById('qstatus').value;
    if (qs && (n.status === 'closed' ? 'closed' : 'open') !== qs) return false;
  }
  if (n.type === 'criterion') {
    const c = document.getElementById('criterion').value;
    if (c && String(!!n.attrs.satisfied) !== (c === 'yes' ? 'true' : 'false')) return false;
  }
  return true;
}

function visibleNodes() {
  let ids = NODES.filter(setVisible).map(n => n.id);
  // Genealogy mode restricts the view to decision nodes.
  if (genealogyMode) {
    ids = ids.filter(id => byId[id] && byId[id].type === 'decision');
  }
  // search
  if (searchText) {
    const hits = computeMatches(searchText);
    searchHits = hits;
    if (hits.size === 0) { return []; }
    ids = ids.filter(id => hits.has(id));
  } else { searchHits = null; }
  // neighborhood
  if (hopFrom) {
    const reach = new Set([hopFrom]);
    let frontier = [hopFrom];
    for (let h = 0; h < hopN; h++) {
      const next = [];
      frontier.forEach(id => {
        Object.keys(adj[id] || {}).forEach(t => { if (!reach.has(t)) { reach.add(t); next.push(t); } });
      });
      frontier = next;
    }
    ids = ids.filter(id => reach.has(id));
  }
  return ids;
}

function visibleEdges(ids) {
  const s = new Set(ids);
  return EDGES.filter(e => (e.source && s.has(e.source) && e.target && s.has(e.target)));
}

function lodFromScale() { return scale < 0.35 ? 'far' : scale < 1.1 ? 'mid' : 'near'; }

