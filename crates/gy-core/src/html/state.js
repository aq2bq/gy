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

const degree = Object.fromEntries(NODES.map(n => [n.id, Object.keys(adj[n.id]).length]));

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
// If even one hop exceeds the threshold, focusedSelection bounds rendering.
function pickHop(start) {
  for (let n = HOP_CAP; n >= 1; n--) {
    const s = reachable(start, n);
    if (s.size <= MODE_THRESHOLD) return {n, size: s.size};
  }
  const s = reachable(start, 1);
  return {n: 1, size: s.size};
}


// URL state is the shared location contract. Pan and zoom are deliberately absent.
let lastLocationState = '';
function locationState(value) {
  const fields = ['scopeSel', 'stateSel', 'qstatus', 'criterion'];
  if (value === undefined) {
    return JSON.stringify({selected, focus: focusHistory, types: typeState,
      filters: Object.fromEntries(fields.map(id => [id, document.getElementById(id).value])),
      search: searchText, radiusChoice: document.getElementById('hopRadius').value, tab: document.querySelector('#tabs .on').dataset.tab, genealogy: genealogyMode});
  }
  focusHistory.splice(0, focusHistory.length, {id:null, radius:null});
  if (Array.isArray(value.focus)) value.focus.forEach(f => {
    if (f && typeof f.id === 'string' && Object.hasOwn(byId, f.id) && Number.isInteger(f.radius) && f.radius >= 1 && f.radius <= HOP_CAP)
      focusHistory.push({id:f.id, radius:f.radius});
  });
  Object.keys(typeState).forEach(k => { typeState[k] = value.types?.[k] !== false; });
  document.querySelectorAll('#typeChips .chip').forEach(c => { c.classList.toggle('on', typeState[c.dataset.kind]); c.setAttribute('aria-pressed', String(typeState[c.dataset.kind])); });
  fields.forEach(id => { document.getElementById(id).value = typeof value.filters?.[id] === 'string' ? value.filters[id] : ''; });
  document.getElementById('hopRadius').value = ['', '1','2','3','4','5'].includes(value.radiusChoice) ? value.radiusChoice : '';
  searchText = typeof value.search === 'string' ? value.search : '';
  document.getElementById('q').value = searchText;
  genealogyMode = value.genealogy === true;
  const tabs = [...document.querySelectorAll('#tabs button')].map(b => b.dataset.tab);
  gotoTab(tabs.includes(value.tab) ? value.tab : 'overview');
  document.getElementById('hopFrom').value = currentFocus().id || '';
  closeClusterPanel(); hideDetail();
  document.getElementById('locationStatus').textContent = '';
  if (typeof value.selected === 'string') {
    if (Object.hasOwn(byId, value.selected)) showDetail(value.selected);
    else document.getElementById('locationStatus').textContent = 'Node not found: ' + value.selected;
  }
}
function locationHash(state) {
  const prefix = 'view=';
  if (state !== undefined) return '#' + prefix + encodeURIComponent(state);
  try {
    const hash = decodeURIComponent(location.hash.slice(1));
    const value = hash.startsWith(prefix) ? JSON.parse(hash.slice(prefix.length)) : (hash ? {selected:hash} : {});
    return value && typeof value === 'object' ? value : {};
  } catch (_) { return {}; }
}
function restoreLocation() {
  const state = locationHash();
  locationState(state);
  lastLocationState = locationState();
  resetView();
}
function saveLocation() {
  const state = locationState();
  if (state === lastLocationState) return;
  history.pushState(null, '', locationHash(state));
  lastLocationState = state;
  document.getElementById('locationStatus').textContent = '';
}
