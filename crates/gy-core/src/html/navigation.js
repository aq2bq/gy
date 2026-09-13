// ---------- focus navigation ----------
function startHop(id) {
  if (!byId[id]) return;
  const radius = focusPlan(id).n;
  if (currentFocus().id !== id || currentFocus().radius !== radius) {
    focusHistory.push({id, radius});
  }
  closeClusterPanel();
  showDetail(id);
  redraw();
}

function returnFocus(index) {
  if (!Number.isInteger(index) || index < 0 || index >= focusHistory.length) return;
  focusHistory.splice(index + 1);
  closeClusterPanel();
  if (currentFocus().id) showDetail(currentFocus().id);
  else hideDetail();
  redraw();
}

function renderNavigation(ids) {
  const focus = currentFocus();
  const path = document.getElementById('focusPath');
  // Keep the path DOM stable during pan/zoom and detail open/close.
  const pathKey = JSON.stringify(focusHistory);
  if (path.dataset.path !== pathKey) {
    path.dataset.path = pathKey;
    path.replaceChildren();
    focusHistory.forEach((entry, index) => {
      if (index) path.appendChild(document.createTextNode(' → '));
      const button = document.createElement('button');
      button.textContent = entry.id ? entry.id + ' · ' + byId[entry.id].title : 'All nodes';
      button.title = button.textContent;
      button.dataset.depth = index;
      if (index === focusHistory.length - 1) button.setAttribute('aria-current', 'location');
      button.addEventListener('click', () => returnFocus(index));
      path.appendChild(button);
    });
    path.lastElementChild.scrollIntoView({block: 'nearest', inline: 'nearest'});
  }
  document.getElementById('focusBack').disabled = focusHistory.length === 1;
  document.getElementById('focusAll').disabled = focusHistory.length === 1;
  document.getElementById('focusDetail').disabled = !focus.id;
  document.getElementById('focusStatus').textContent = focus.id
    ? 'Focus: ' + focus.id + ' · ' + focus.radius + ' hops' +
      (Object.keys(adj[focus.id] || {}).length ? '' : ' · No connections in this graph') +
      (ids.includes(focus.id) ? '' : ' · Focus hidden by current filters, search, or lineage')
    : 'All nodes · no focus';
  document.getElementById('displayStatus').textContent =
    'Types: ' + (Object.keys(typeState).filter(k => typeState[k]).join(', ') || 'none') +
    ' · Scope: ' + (scopeSel.value || 'all') + ' · State: ' + (stateSel.value || 'any') +
    ' · Questions: ' + (document.getElementById('qstatus').value || 'any') +
    ' · Criteria: ' + (document.getElementById('criterion').value || 'any') +
    ' · Search: ' + (searchText || '(none)') + ' · Genealogy: ' + (genealogyMode ? 'on' : 'off');
  document.getElementById('applyHop').textContent = focusLabel(focus.id || '');
  document.getElementById('hopFrom').value = focus.id || '';
  document.getElementById('hopN').textContent = focus.id ? focus.radius + ' hops in current view' : 'Automatic radius';
  document.getElementById('genealogy').setAttribute('aria-pressed', String(genealogyMode));
  document.getElementById('genealogy').style.borderColor = genealogyMode ? 'var(--hl)' : '';
}

document.getElementById('focusBack').addEventListener('click', () => returnFocus(focusHistory.length - 2));
document.getElementById('focusAll').addEventListener('click', () => returnFocus(0));
document.getElementById('focusDetail').addEventListener('click', () => {
  if (currentFocus().id) { showDetail(currentFocus().id); redraw(); }
});
