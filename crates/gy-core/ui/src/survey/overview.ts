import { esc } from '../components';
import { D, lintArr, states } from '../data';
import { element } from '../dom';
import { setStateFilter } from '../filters';
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
    element('acSatisfied').textContent = D.criteria.satisfied + ' / ' + D.criteria.total;
    element('acTotal').textContent = D.criteria.satisfied;
    element('openQ').textContent = openQs.length;
    element('waitRefs').textContent = waitRefs;
    element('nextCount').textContent = (D.next || []).length;
    element('lintErr').textContent = lintErr;
    element('lintWarn').textContent = lintWarn;
    // state bars
    const maxState = Math.max(1, ...Object.values(states));
    const stateColors = [token('--requirement'), token('--need'), token('--gate'), token('--decision'), token('--question'), token('--color-c97b4f'), token('--criterion'), token('--color-a8608a'), token('--color-7d6c5f'), token('--color-556b7a'), token('--color-3f8f6b')];
    let i = 0;
    const stateBar = element('stateBars');
    Object.entries(states).forEach(([st, cnt]) => {
        const w = Math.round(cnt / maxState * 100);
        const d = document.createElement('div');
        d.className = 'bar';
        d.style.setProperty('--w', w + '%');
        d.style.setProperty('--bar', stateColors[i++ % stateColors.length]);
        d.title = st;
        d.innerHTML = esc(st) + ' <em>' + cnt + '</em>';
        d.addEventListener('click', () => { setStateFilter(st); });
        stateBar.appendChild(d);
    });
}
