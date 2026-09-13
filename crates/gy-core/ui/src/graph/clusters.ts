import { byId } from '../data';
import { element } from '../dom';
import { renderClusterRows } from '../list';
import { adj, currentFocus } from '../state';
// ---------- cluster panel (H25) ----------
export const panel = element('clusterPanel');
export function openClusterPanel(g, members) {
    const [scope, type] = g.split('\u0000');
    openNodePanel(members, type, scope, g);
}
export function openOmittedPanel(members) {
    panel.dataset.members = JSON.stringify(members);
    openNodePanel(members, 'omitted nodes', currentFocus().id, 'omitted');
}
export function openNodePanel(members, type, scope, g) {
    panel.classList.add('on');
    panel.dataset.g = g;
    const rows = members.map(id => {
        const n = byId[id];
        const deg = Object.keys(adj[id] || {}).length;
        return { id, title: n ? n.title : '', deg, alive: n && n.type === 'decision' && !(n.superseded_by && n.superseded_by.length) };
    });
    renderClusterRows(rows, type, scope, g);
}
export function closeClusterPanel() { panel.classList.remove('on'); }
