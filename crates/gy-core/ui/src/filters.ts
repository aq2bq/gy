import { redraw } from './components';
import { D, NODES, byId, states } from './data';
import { hideDetail } from './detail/index';
import { element } from './dom';
import { closeClusterPanel } from './graph/clusters';
import { resetView, zoomBy } from './graph/viewport';
import { startHop } from './navigation';
import { focusLabel, genealogyMode, selected, setActiveTab, setFilter, setGenealogyMode, setLod, setRadiusChoice, setSearchText, setType, truncateFocus, typeState } from './state';
import { KIND_COLORS } from './tokens';
export const typeChips = element('typeChips');
export const scopeSel = element('scopeSel');
export const stateSel = element('stateSel');
export function initFilters() {
    // ---------- filters ----------
    (['need', 'question', 'decision', 'requirement', 'criterion', 'gate'] as const).forEach(k => {
        const chip = document.createElement('button');
        chip.type = 'button';
        chip.setAttribute('aria-pressed', 'true');
        chip.className = 'chip on';
        chip.dataset.kind = k;
        chip.style.borderColor = KIND_COLORS[k];
        chip.textContent = k;
        chip.addEventListener('click', () => {
            setType(k as keyof typeof typeState, !typeState[k]);
            chip.classList.toggle('on', typeState[k]);
            chip.setAttribute('aria-pressed', String(typeState[k]));
            redraw();
        });
        typeChips.appendChild(chip);
    });
    (D.scopes || []).forEach(s => {
        const o = document.createElement('option');
        o.value = s;
        o.textContent = s;
        scopeSel.appendChild(o);
    });
    Object.keys(states).forEach(st => {
        const o = document.createElement('option');
        o.value = st;
        o.textContent = st;
        stateSel.appendChild(o);
    });
    (['scopeSel', 'stateSel', 'qstatus', 'criterion'] as const).forEach(id => element(id).addEventListener('change', () => { setFilter(id, element(id).value); redraw(); }));
    element('q').addEventListener('input', () => { setSearchText(element('q').value.trim()); redraw(); });
    element('clearFilter').addEventListener('click', () => {
        Object.keys(typeState).forEach(k => { setType(k as keyof typeof typeState, true); });
        document.querySelectorAll<HTMLElement>('#typeChips .chip').forEach(c => { c.classList.add('on'); c.setAttribute('aria-pressed', 'true'); });
        scopeSel.value = '';
        stateSel.value = '';
        element('qstatus').value = '';
        element('criterion').value = '';
        element('q').value = '';
        setSearchText('');
        (['scopeSel', 'stateSel', 'qstatus', 'criterion'] as const).forEach(id => setFilter(id, ''));
        setRadiusChoice('');
        truncateFocus(1);
        hideDetail();
        setGenealogyMode(false);
        element('hopRadius').value = '';
        element('hopFrom').value = '';
        closeClusterPanel();
        redraw();
    });
    element('hopFrom').addEventListener('input', () => { element('applyHop').textContent = focusLabel(element('hopFrom').value.trim()); });
    element('hopRadius').addEventListener('change', () => {
        setRadiusChoice(element('hopRadius').value);
        element('applyHop').textContent = focusLabel(element('hopFrom').value.trim());
        if (selected)
            element('detailFocus').textContent = focusLabel(selected);
    });
    element('applyHop').addEventListener('click', () => {
        const id = element('hopFrom').value.trim();
        if (!byId[id]) {
            alert('Unknown node id: ' + id);
            return;
        }
        startHop(id);
    });
    element('genealogy').addEventListener('click', () => {
        setGenealogyMode(!genealogyMode);
        if (genealogyMode) {
            setLod('near');
        }
        closeClusterPanel();
        redraw();
    });
    element('zin').addEventListener('click', () => zoomBy(1.6));
    element('zout').addEventListener('click', () => zoomBy(1 / 1.6));
    element('zfit').addEventListener('click', resetView);
}
export function setStateFilter(st) { stateSel.value = st; setFilter('stateSel', stateSel.value); redraw(); }
export function gotoTab(t) {
    setActiveTab(t);
    document.querySelectorAll<HTMLElement>('#tabs button').forEach(b => b.classList.toggle('on', b.dataset.tab === t));
    document.querySelectorAll<HTMLElement>('.page').forEach(p => p.classList.toggle('on', p.id === 'page-' + t));
}
// ---------- search index ----------
export function haystack(n) {
    let parts = [n.id, n.title, n.type, n.scope, n.status || ''];
    for (const [k, v] of Object.entries(n.attrs))
        parts.push(k + '=' + String(v));
    parts.push(n.body || '');
    return parts.join('\n').toLowerCase();
}
export function computeMatches(q) {
    if (!q)
        return null;
    const lq = q.toLowerCase();
    return new Set(NODES.filter(n => haystack(n).includes(lq)).map(n => n.id));
}
