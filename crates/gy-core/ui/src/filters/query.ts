import { NODES } from '../data';
import { filterState, searchText, setSearchHits, typeState } from '../state';
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
export function setVisible(n) {
    if (typeState[n.type] !== true)
        return false;
    if (filterState.scopeSel && n.scope !== filterState.scopeSel)
        return false;
    if (n.type === 'requirement') {
        if (filterState.stateSel && (n.state || '') !== filterState.stateSel)
            return false;
    }
    if (n.type === 'question') {
        const qs = filterState.qstatus;
        if (qs && (n.status === 'closed' ? 'closed' : 'open') !== qs)
            return false;
    }
    if (n.type === 'criterion') {
        const c = filterState.criterion;
        if (c && String(!!n.attrs.satisfied) !== (c === 'yes' ? 'true' : 'false'))
            return false;
    }
    return true;
}

let previousKey = '';
let matches = [];
export function filteredNodes() {
    const key = JSON.stringify([typeState, filterState, searchText]);
    if (key !== previousKey) {
        const hits = computeMatches(searchText);
        setSearchHits(hits);
        matches = NODES.filter(n => setVisible(n) && (!hits || hits.has(n.id)));
        previousKey = key;
    }
    return matches;
}
