// ---------- filters ----------
const typeState = {};
const typeChips = document.getElementById('typeChips');
['need','question','decision','requirement','criterion','gate'].forEach(k => {
  typeState[k] = true;
  const chip = document.createElement('div');
  chip.className = 'chip on';
  chip.dataset.kind = k;
  chip.style.borderColor = KIND_COLORS[k];
  chip.textContent = k;
  chip.addEventListener('click', () => {
    typeState[k] = !typeState[k];
    chip.classList.toggle('on', typeState[k]);
    redraw();
  });
  typeChips.appendChild(chip);
});
const scopeSel = document.getElementById('scopeSel');
(D.scopes || []).forEach(s => {
  const o = document.createElement('option');
  o.value = s; o.textContent = s; scopeSel.appendChild(o);
});
const stateSel = document.getElementById('stateSel');
Object.keys(states).forEach(st => {
  const o = document.createElement('option');
  o.value = st; o.textContent = st; stateSel.appendChild(o);
});
scopeSel.addEventListener('change', redraw);
stateSel.addEventListener('change', redraw);
document.getElementById('qstatus').addEventListener('change', redraw);
document.getElementById('criterion').addEventListener('change', redraw);
document.getElementById('q').addEventListener('input', () => { searchText = document.getElementById('q').value.trim(); redraw(); });
document.getElementById('clearFilter').addEventListener('click', () => {
  Object.keys(typeState).forEach(k => { typeState[k] = true; });
  document.querySelectorAll('#typeChips .chip').forEach(c => c.classList.add('on'));
  scopeSel.value = ''; stateSel.value = ''; document.getElementById('qstatus').value = '';
  document.getElementById('criterion').value = '';
  document.getElementById('q').value = '';
  searchText = '';
  hopFrom = null; hopN = 1; hopInfo = null;
  genealogyMode = false;
  closeClusterPanel();
  redraw();
});
document.getElementById('applyHop').addEventListener('click', () => {
  const id = document.getElementById('hopFrom').value.trim();
  if (!byId[id]) { alert('Unknown node id: ' + id); return; }
  startHop(id);
});
document.getElementById('genealogy').addEventListener('click', () => {
  genealogyMode = !genealogyMode;
  document.getElementById('genealogy').style.borderColor = genealogyMode ? 'var(--hl)' : '';
  if (genealogyMode) {
    lod = 'near';
  }
  closeClusterPanel();
  resetView();
  redraw();
});
document.getElementById('zin').addEventListener('click', () => zoomBy(1.6));
document.getElementById('zout').addEventListener('click', () => zoomBy(1/1.6));
document.getElementById('zfit').addEventListener('click', resetView);

function setStateFilter(st) { stateSel.value = st; redraw(); }
function gotoTab(t) {
  document.querySelectorAll('#tabs button').forEach(b => b.classList.toggle('on', b.dataset.tab === t));
  document.querySelectorAll('.page').forEach(p => p.classList.toggle('on', p.id === 'page-' + t));
}

// ---------- search index ----------
function haystack(n) {
  let parts = [n.id, n.title, n.type, n.scope, n.status || ''];
  for (const [k, v] of Object.entries(n.attrs)) parts.push(k + '=' + String(v));
  parts.push(n.body || '');
  return parts.join('\n').toLowerCase();
}
function computeMatches(q) {
  if (!q) return null;
  const lq = q.toLowerCase();
  return new Set(NODES.filter(n => haystack(n).includes(lq)).map(n => n.id));
}

