import { EDGES, NODES, byId } from '../data';
import { filteredNodes } from '../filters/query';
import { MODE_THRESHOLD, adj, currentFocus, degree, filterState, genealogyMode, reachable, scale, searchText, setSearchHits, typeState } from '../state';
export function candidateNodes() {
    let ids = filteredNodes().map(n => n.id);
    // Genealogy mode restricts the view to decision nodes.
    if (genealogyMode) {
        ids = ids.filter(id => byId[id] && byId[id].type === 'decision');
    }
    // neighborhood
    const focus = currentFocus();
    if (focus.id) {
        const reach = reachable(focus.id, focus.radius);
        ids = ids.filter(id => reach.has(id));
    }
    return ids;
}
export function visibleEdges(ids) {
    const s = new Set(ids);
    return EDGES.filter(e => (e.source && s.has(e.source) && e.target && s.has(e.target)));
}
export function lodFromScale() { return scale < 0.35 ? 'far' : scale < 1.1 ? 'mid' : 'near'; }
// Bound focused rendering without changing the filtered neighborhood or history.
let selectionKey = '', selectionCache = null;
export function focusedSelection() {
    const candidates = candidateNodes();
    const focus = currentFocus();
    if (!focus.id || candidates.length <= MODE_THRESHOLD)
        return { ids: candidates, omitted: [] };
    const key = JSON.stringify([focus.id, focus.radius, candidates]);
    if (key === selectionKey)
        return selectionCache;
    const distance = new Map([[focus.id, 0]]), queue = [focus.id];
    for (let i = 0; i < queue.length; i++) {
        const id = queue[i], d = distance.get(id);
        if (d >= focus.radius)
            continue;
        for (const neighbor of Object.keys(adj[id] || {})) {
            if (!distance.has(neighbor)) {
                distance.set(neighbor, d + 1);
                queue.push(neighbor);
            }
        }
    }
    const ranked = [...candidates].sort((a, b) => distance.get(a) - distance.get(b) || degree[b] - degree[a] || (a < b ? -1 : a > b ? 1 : 0));
    selectionKey = key;
    selectionCache = { ids: ranked.slice(0, MODE_THRESHOLD), omitted: ranked.slice(MODE_THRESHOLD) };
    return selectionCache;
}
export function visibleNodes() { return focusedSelection().ids; }
