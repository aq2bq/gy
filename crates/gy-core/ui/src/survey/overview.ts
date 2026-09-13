import { esc } from '../components';
import { D, lintArr, states } from '../data';
import { element } from '../dom';
import { locationHash, typeState } from '../state';
import { token } from '../tokens';
export function initOverview() {
    // header meta
    element('metaGen').textContent = 'Generated: ' + D.generated_at;
    element('metaScope').textContent = 'Scopes: ' + (D.scope ? D.scope : 'all (' + (D.scopes || []).join(', ') + ')');
    element('metaNodes').textContent = 'Nodes: ' + D.node_count;
    element('integrityNote').textContent = D.integrity_note;
    // ---------- overview numbers ----------
    const lintErr = lintArr.filter(d => d.severity === 'error').length;
    const lintWarn = lintArr.filter(d => d.severity === 'warn').length;
    const openQs = (D.open_questions || []);
    const waitRefs = openQs.reduce((a, q) => a + (q.referencing || []).length, 0);
    element('acSatisfied').textContent = D.criteria.satisfied;
    element('acTotal').textContent = D.criteria.total;
    element('openQ').textContent = openQs.length;
    element('waitRefs').textContent = waitRefs;
    element('nextCount').textContent = (D.next || []).length;
    element('lintErr').textContent = lintErr;
    element('lintWarn').textContent = lintWarn;
    function target(kind = '', filters = {}, overview = '', tab = 'overview') {
        return locationHash(JSON.stringify({types: Object.fromEntries(Object.keys(typeState).map(k => [k, !kind || k === kind])), filters, overview, tab}));
    }
    function link(id, href) {
        const box = element(id).parentElement;
        const a = document.createElement('a');
        a.className = box.className;
        if (box.id) a.id = box.id;
        a.href = href;
        a.append(...box.childNodes);
        box.replaceWith(a);
    }
    link('acSatisfied', target('criterion', {criterion:'yes'}));
    link('acTotal', target('criterion'));
    link('openQ', target('question', {qstatus:'open'}));
    link('nextCount', target('need', {}, 'next'));
    link('lintErr', target('', {}, 'lint-error', 'jams'));
    link('lintWarn', target('', {}, 'lint-warn', 'jams'));
    // Waiting references count relationships, not distinct table records.
    element('waitRefs').parentElement.classList.add('text-stat');
    // state bars
    const maxState = Math.max(1, ...Object.values(states));
    const stateColors = [token('--requirement'), token('--need'), token('--gate'), token('--decision'), token('--question'), token('--color-c97b4f'), token('--criterion'), token('--color-a8608a'), token('--color-7d6c5f'), token('--color-556b7a'), token('--color-3f8f6b')];
    let i = 0;
    const stateBar = element('stateBars');
    Object.entries(states).forEach(([st, cnt]) => {
        const w = Math.round(cnt / maxState * 100);
        const d = document.createElement('a');
        d.className = 'bar';
        d.href = target('requirement', {stateSel:st});
        d.style.setProperty('--w', w + '%');
        d.style.setProperty('--bar', stateColors[i++ % stateColors.length]);
        d.title = st;
        d.innerHTML = esc(st) + ' <em>' + cnt + '</em>';

        stateBar.appendChild(d);
    });
}
