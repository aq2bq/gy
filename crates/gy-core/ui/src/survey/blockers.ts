import { locationHash, overviewSource } from '../state';
import { esc } from '../components';
import { D, lintArr } from '../data';
import { selectNode } from '../detail/index';
import { element } from '../dom';
// ---------- blockers (jam) ----------
export function jamList(ulEl, items, tag) {
    const ul = element(ulEl);
    ul.innerHTML = '';
    items.forEach(it => {
        const li = document.createElement('li');
        const a = document.createElement('a');
        a.dataset.go = it.id;
        a.href = locationHash(JSON.stringify({selected:it.id}));
        a.innerHTML = '<span class="id">' + esc(it.id) + '</span> <span class="lbl">' + esc(it.label) + '</span>';
        li.appendChild(a);
        ul.appendChild(li);
    });
    if (!items.length) {
        ul.innerHTML = '<li class="small">none</li>';
    }
}
// open questions
export function initBlockers() {
    jamList('jamQ', (D.open_questions || []).flatMap(q => [
        { id: q.id, label: (q.title || '') + (q.referencing && q.referencing.length ? ' — referenced by ' + q.referencing.join(', ') : '') }
    ]), 'q');
    // next needs
    jamList('jamNext', (D.next || []), 'next');
    // missing
    jamList('jamMissing', (D.missing || []).map(m => ({ id: m.id, label: 'next_evidence ' + (m.next_evidence ? 'set' : 'empty') + '; responsible ' + (m.responsible ? 'set' : 'empty') })), 'missing');
    // dangling
    jamList('jamDangling', (D.dangling || []).map(d => ({ id: d.source, label: 'references missing node ' + d.target })), 'dangling');
    renderLint();
    element('left').addEventListener('click', (e) => {
        const t = (e.target as Element).closest<HTMLElement>('[data-go]');
        if (t && !e.ctrlKey && !e.metaKey && !e.shiftKey && !e.altKey) {
            e.preventDefault(); selectNode(t.dataset.go);
        }
    });
}
export function renderLint() {
    const lintUl = element('jamLint');
    lintUl.innerHTML = '';
    const rows = lintArr.filter(d => !overviewSource.startsWith('lint-') || d.severity === overviewSource.slice(5));
    rows.forEach(d => {
        const li = document.createElement('li');
        const a = document.createElement('a');
        a.dataset.go = d.id;
        a.href = locationHash(JSON.stringify({selected:d.id}));
        a.innerHTML = '<span class="lint-sev" style="color:' + (d.severity === 'error' ? 'var(--err)' : 'var(--warn)') + '">' + esc(d.severity) + '</span> ' +
            '<span class="id">' + esc(d.rule) + '</span> <span class="lbl">' + esc(d.id) + ' — ' + esc(d.message) + '</span>';
        li.appendChild(a);
        lintUl.appendChild(li);
    });
    if (!rows.length)
        lintUl.innerHTML = '<li class="small">no findings</li>';
}
