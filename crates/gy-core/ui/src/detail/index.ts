import { esc, redraw } from '../components';
import { DEP, EDGES, byId } from '../data';
import { element } from '../dom';
import { selected, setSelected } from '../state';
import { renderBody } from './markdown';
// ---------- detail panel ----------
export function relFor(id) {
    const out = [];
    EDGES.forEach(e => {
        if (e.source === id)
            out.push({ other: e.target, label: e.label, dir: '→' });
        if (e.target === id)
            out.push({ other: e.source, label: e.reverse, dir: '←' });
    });
    // preserved order: show=neighbors order uses EDGES table; fine to sort by other id
    return out;
}
// Source HTML is always text. Only renderer-owned tags enter the detail DOM.
export const DETAIL_FIELDS = {
    created: 'Created', closed_at: 'Closed', satisfied_at: 'Satisfied at',
    parent_issue: 'Parent issue', pr_url: 'Pull request', responsible: 'Responsible',
    decider: 'Decider', next_evidence: 'Next evidence', closure_note: 'Closure note',
    evidence: 'Evidence', summary: 'Summary', contracts_changed: 'Contracts changed',
    artifacts: 'Artifacts', production: 'Production', deviations: 'Deviations',
    residual: 'Residual', compressed_from: 'Archived record'
};
export const LINEAGE_LABELS = new Set(['narrows', 'narrowed-by', 'widens', 'widened-by',
    'supersedes', 'superseded-by', 'completes', 'completed-by']);
export function detailValue(value) { return typeof value === 'string' ? value : JSON.stringify(value, null, 2); }
export function valueHTML(value) {
    const text = detailValue(value);
    // A URL is a link only when the entire scalar is an HTTP(S) URL.
    if (typeof value === 'string' && /^https?:\/\/[^\s]+$/i.test(value)) {
        return '<a href="' + esc(value) + '" target="_blank" rel="noopener noreferrer">' + esc(value) + '</a>';
    }
    return esc(text);
}
export function nodeLink(id) {
    return byId[id] ? '<button class="node-link" data-go="' + esc(id) + '">' + esc(id) + '</button>' : esc(id);
}
export function detailBadge(field, label, value, tone = '') {
    return '<span class="detail-badge ' + tone + '" data-field="' + field + '"><span class="badge-label">' + label + '</span> ' + esc(value) + '</span>';
}
export function proseClass(text) {
    // Script-sensitive measure, not a language or domain-state inference.
    const letters = String(text).match(/\p{L}/gu) || [];
    const cjk = String(text).match(/[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}]/gu) || [];
    return cjk.length > letters.length / 2 ? 'prose-ja' : 'prose-latin';
}
export function selectNode(id) { showDetail(id); redraw(); }
export function showDetail(id) {
    const n = byId[id];
    if (!n)
        return;
    const changed = selected !== id;
    setSelected(id);
    element('hopFrom').value = id;
    const consumed = new Set(['id', 'type', 'scope', 'title']);
    let badges = detailBadge('type', 'Type', n.type, 'kind-' + n.type) + detailBadge('scope', 'Scope', n.scope);
    if (typeof n.attrs.status === 'string') {
        badges += detailBadge('status', 'Status', n.state || n.status);
        if (n.state && n.status !== n.state)
            badges += '<span class="status-note">' + esc(n.status) + '</span>';
        consumed.add('status');
    }
    if (n.type === 'criterion') {
        badges += detailBadge('satisfied', 'Acceptance', n.attrs.satisfied === true ? 'Satisfied' : 'Not satisfied', n.attrs.satisfied === true ? 'positive' : '');
        if (typeof n.attrs.satisfied === 'boolean')
            consumed.add('satisfied');
    }
    if (n.type === 'question' && typeof n.attrs.closed_by === 'string') {
        badges += detailBadge('closed_by', 'Closed by', n.attrs.closed_by);
        consumed.add('closed_by');
    }
    const successors = n.superseded_by || [];
    if (n.type === 'decision')
        badges += detailBadge('superseded', 'Decision', successors.length ? 'Superseded' : 'Current', successors.length ? 'superseded' : 'positive');
    let declarations = '';
    for (const [key, label] of Object.entries(DETAIL_FIELDS)) {
        if (!Object.prototype.hasOwnProperty.call(n.attrs, key))
            continue;
        consumed.add(key);
        const value = n.attrs[key];
        const rendered = ['created', 'closed_at', 'satisfied_at'].includes(key) && typeof value === 'string'
            ? '<time datetime="' + esc(value) + '">' + esc(value) + '</time>' : valueHTML(value);
        declarations += '<div class="declaration" data-field="' + key + '"><dt>' + label + '</dt><dd>' + rendered + '</dd></div>';
    }
    let scope = '';
    if (Object.prototype.hasOwnProperty.call(n.attrs, 'decision_scope')) {
        consumed.add('decision_scope');
        const text = detailValue(n.attrs.decision_scope);
        scope = '<section id="detailScope"><h2>Applicability</h2><div class="detail-prose ' + proseClass(text) + '">' + renderBody(text) + '</div></section>';
    }
    let additional = '';
    for (const [key, value] of Object.entries(n.attrs)) {
        if (consumed.has(key))
            continue;
        additional += '<div class="additional-attribute" data-key="' + esc(key) + '"><dt>' + esc(key) + '</dt><dd><pre>' + esc(detailValue(value)) + '</pre></dd></div>';
    }
    const relations = relFor(id);
    const group = (title, items) => items.length ? '<section class="detail-relations"><h2>' + title + '</h2>' + items.map(r => '<div class="rel"><span class="relation-chip">' + esc(r.dir + ' ' + r.label) + '</span> ' + nodeLink(r.other) + '</div>').join('') + '</section>' : '';
    const dependencies = DEP.filter(d => d.requirement === id);
    const depHTML = dependencies.length ? '<section id="detailDependencies"><h2>Decision dependencies</h2><table><thead><tr><th>Decision</th><th>Role</th><th>Superseded by</th></tr></thead><tbody>' +
        dependencies.map(d => '<tr><td>' + nodeLink(d.decision) + '</td><td>' + esc(d.role) + '</td><td>' + (d.superseded_by || []).map(nodeLink).join(', ') + '</td></tr>').join('') + '</tbody></table></section>' : '';
    let displayBody = n.body || '';
    const marks = n.body_marks || [];
    marks.forEach(m => {
        if (m.found && m.mark)
            displayBody = displayBody.split(m.mark).join(m.mark + ' ⟦' + m.label + ': ' + m.source + '⟧');
    });
    const markHTML = marks.length ? '<section id="detailMarks"><h2>Affected passages</h2>' + marks.map(m => '<div class="passage-note" data-found="' + String(m.found) + '"><span class="relation-chip">' + esc(m.label) + '</span> ' + nodeLink(m.source) +
        (m.mark ? '<blockquote>' + esc(m.mark) + '</blockquote><p>' + (m.found ? 'Found in the source body.' : 'Location in body could not be found.') + '</p>' : '<p>No mark identifies the affected passage.</p>') + '</div>').join('') + '</section>' : '';
    element('detailBody').innerHTML =
        '<div class="detail-heading"><div class="detail-id">' + esc(n.id) + '</div><h3>' + esc(n.title) + '</h3><div class="detail-badges">' + badges + '</div>' +
            (successors.length ? '<div class="successors">Superseded by ' + successors.map(nodeLink).join(', ') + '</div>' : '') + '</div>' + scope +
            '<section id="detailContent"><h2>Body</h2><div id="body" class="detail-prose ' + proseClass(n.body || '') + '">' + renderBody(displayBody) + '</div></section>' + markHTML +
            (declarations ? '<section id="detailDeclarations"><h2>Declarations</h2><dl>' + declarations + '</dl></section>' : '') +
            group('Decision lineage', relations.filter(r => LINEAGE_LABELS.has(r.label))) +
            group('Relationships', relations.filter(r => !LINEAGE_LABELS.has(r.label))) + depHTML +
            (additional ? '<section id="detailAdditional"><h2>Additional attributes</h2><dl>' + additional + '</dl></section>' : '');
    const detail = element('detail');
    detail.classList.add('on');
    if (changed)
        detail.scrollTop = 0;
}
export function hideDetail() {
    element('detail').classList.remove('on');
    setSelected(null);
}
export function closeDetail() { hideDetail(); redraw(); }
export function initDetail() {
    element('closeDetail').addEventListener('click', closeDetail);
    element('detail').addEventListener('click', (e) => {
        const t = (e.target as Element).closest<HTMLElement>('[data-go]');
        if (t)
            selectNode(t.dataset.go);
    });
}
