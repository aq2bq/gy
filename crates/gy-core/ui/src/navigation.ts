import { redraw } from './components';
import { byId } from './data';
import { hideDetail, showDetail } from './detail/index';
import { element } from './dom';
import { scopeSel, stateSel } from './filters';
import { selected, overviewSource, adj, currentFocus, enterFocus, focusHistory, focusLabel, focusPlan, genealogyMode, searchText, truncateFocus, typeState } from './state';
// ---------- focus navigation ----------
export function startHop(id) {
    if (!byId[id])
        return;
    const radius = focusPlan(id).n;
    if (currentFocus().id !== id || currentFocus().radius !== radius) {
        enterFocus(id, radius);
    }
    showDetail(id);
    redraw();
}
export function returnFocus(index) {
    if (!Number.isInteger(index) || index < 0 || index >= focusHistory.length)
        return;
    truncateFocus(index + 1);
    if (currentFocus().id)
        showDetail(currentFocus().id);
    else
        hideDetail();
    redraw();
}
export function renderNavigation(ids) {
    const focus = currentFocus();
    const path = element('focusPath');
    // Keep the path DOM stable during pan/zoom and detail open/close.
    const pathKey = JSON.stringify(focusHistory);
    if (path.dataset.path !== pathKey) {
        path.dataset.path = pathKey;
        path.replaceChildren();
        focusHistory.forEach((entry, index) => {
            if (index)
                path.appendChild(document.createTextNode(' → '));
            const button = document.createElement('button');
            button.textContent = entry.id ? entry.id : 'All nodes';
            button.title = entry.id ? entry.id + ' · ' + byId[entry.id].title : 'All nodes';
            button.dataset.depth = String(index);
            if (index === focusHistory.length - 1)
                button.setAttribute('aria-current', 'location');
            button.addEventListener('click', () => returnFocus(index));
            path.appendChild(button);
        });
        path.lastElementChild.scrollIntoView({ block: 'nearest', inline: 'nearest' });
    }
    element('focusBack').disabled = focusHistory.length === 1;
    element('focusAll').disabled = focusHistory.length === 1;
    element('focusDetail').disabled = !focus.id;
    element('focusNote').textContent = focus.id
        ? ' · ' + focus.radius + (focus.radius === 1 ? ' hop' : ' hops') +
          (Object.keys(adj[focus.id] || {}).length ? '' : ' · No connections in this graph') +
          (ids.includes(focus.id) ? '' : ' · Focus hidden by current filters, search, or lineage') : '';
    const active = [
        overviewSource ? 'Source: ' + overviewSource : '',
        Object.values(typeState).every(Boolean) ? '' : Object.keys(typeState).filter(k => typeState[k]).join(', ') || 'No types',
        scopeSel.value ? 'Scope: ' + scopeSel.value : '',
        stateSel.value ? 'State: ' + stateSel.value : '',
        element('qstatus').value ? 'Questions: ' + element('qstatus').value : '',
        element('criterion').value ? 'Criteria: ' + element('criterion').value : '',
        searchText ? 'Search: ' + searchText : ''
    ].filter(Boolean);
    element('displayStatus').textContent = active.length ? active.length + (active.length === 1 ? ' filter' : ' filters') : 'No filters';
    element('displayStatus').title = active.join(' · ');
    const input = element('hopFrom'), focusKey = JSON.stringify(focus);
    if (input.dataset.focus !== focusKey) {
        input.dataset.focus = focusKey;
        input.value = selected || focus.id || '';
    }
    element('applyHop').textContent = focusLabel(input.value.trim());
    element('genealogy').textContent = genealogyMode ? 'Lineage: on' : 'Lineage: off';
    element('genealogy').setAttribute('aria-pressed', String(genealogyMode));
    element('genealogy').style.borderColor = genealogyMode ? 'var(--hl)' : '';
}
export function initNavigation() {
    element('focusBack').addEventListener('click', () => returnFocus(focusHistory.length - 2));
    element('focusAll').addEventListener('click', () => returnFocus(0));
    element('focusDetail').addEventListener('click', () => {
        if (currentFocus().id) {
            showDetail(currentFocus().id);
            redraw();
        }
    });
}
