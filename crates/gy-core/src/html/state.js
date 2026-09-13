// ---------- graph state ----------
const MODE_THRESHOLD = 60;      // above this: overview clusters; at/below: individual force layout
const HOP_CAP = 5;              // adaptive neighborhood ceiling, still clamped to the threshold
const EDGE_LABEL_MAX = 20;      // draw edge labels only when the individual set is this small
const MAX_LABEL_W = 190;        // graph units: label width budget at near zoom

let searchText = '';
let searchHits = null;
// Navigation history is independent of filters, search, and genealogy.
const focusHistory = [{id: null, radius: null}];
function currentFocus() { return focusHistory[focusHistory.length - 1]; }
let genealogyMode = false;
let selected = null;
let scale = 1, translate = {x:0, y:0};
let viewSource = 'fit';
let lod = 'far';
let positions = {};           // id -> {x,y}, overview cluster-member grid
let drawNodes = [];           // legacy alias for the current visible subset
let genealogyLayout = {};     // id -> {x,y}, genealogy (unchanged by H24)
let forceLayout = {};         // id -> {x,y}, individual mode layout
let forceKey = '';            // cache key for the current force layout
const groupPos = {};

// adjacency (undirected) built once from the edges the graph actually draws
const adj = {};
NODES.forEach(n => adj[n.id] = {});
EDGES.forEach(e => {
  (adj[e.source] = adj[e.source] || {})[e.target] = e.label;
  (adj[e.target] = adj[e.target] || {})[e.source] = e.reverse;
});

// ---------- adaptive neighborhood (H27) ----------
function reachable(start, n) {
  let seen = new Set([start]);
  let frontier = [start];
  for (let i = 0; i < n; i++) {
    const next = [];
    frontier.forEach(id => Object.keys(adj[id] || {}).forEach(t => { if (!seen.has(t)) { seen.add(t); next.push(t); } }));
    frontier = next;
  }
  return seen;
}
// Largest n in [1..HOP_CAP] whose reachable set fits the threshold.
// 1 hop is always degree+1 <= 60, so the rule always terminates.
function pickHop(start) {
  for (let n = HOP_CAP; n >= 1; n--) {
    const s = reachable(start, n);
    if (s.size <= MODE_THRESHOLD) return {n, size: s.size};
  }
  const s = reachable(start, 1);
  return {n: 1, size: s.size};
}

