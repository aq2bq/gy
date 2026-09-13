// ---------- detail panel ----------
function relFor(id) {
  const out = [];
  EDGES.forEach(e => {
    if (e.source === id) out.push({other: e.target, label: e.label, dir: '→'});
    if (e.target === id) out.push({other: e.source, label: e.reverse, dir: '←'});
  });
  // preserved order: show=neighbors order uses EDGES table; fine to sort by other id
  return out;
}

function renderBody(md) {
  // minimal, safe markdown: escape everything, then apply fenced code, headings, lists, paragraphs
  const s = String(md || '').replace(/[\r\n]+$/, '');
  const lines = s.split(/\n/);
  // code fences first
  const out = [];
  let i = 0, fenceOpen = null;
  while (i < lines.length) {
    const line = lines[i];
    const m = line.match(/^```/);
    if (fenceOpen) {
      if (m) { fenceOpen = null; out.push(null); } else { out.push('\u0000code\u0000' + line); }
      i++; continue;
    } else if (m) { fenceOpen = true; out.push(null); i++; continue; }
    out.push(line); i++;
  }
  const html = [];
  let inList = false, inQuote = false;
  out.forEach(line => {
    if (line === null) { if (inList) { html.push('</ul>'); inList = false; } if (inQuote) { html.push('</blockquote>'); inQuote = false; } return; }
    if (line.startsWith('\u0000code\u0000')) {
      if (inList) { html.push('</ul>'); inList = false; }
      html.push('<pre><code>' + esc(line.slice(6)) + '</code></pre>');
      return;
    }
    const h = line.match(/^(#{1,4})\s+(.*)$/);
    if (h) {
      if (inList) { html.push('</ul>'); inList = false; }
      if (inQuote) { html.push('</blockquote>'); inQuote = false; }
      html.push('<h4>' + esc(h[2]) + '</h4>');
      return;
    }
    const li = line.match(/^\s*[-*+]\s+(.*)$/);
    if (li) { if (!inList) { html.push('<ul>'); inList = true; } html.push('<li>' + inline(li[1]) + '</li>'); return; }
    const bq = line.match(/^\s*>\s?(.*)$/);
    if (bq) { if (!inQuote) { html.push('<blockquote>'); inQuote = true; } html.push(inline(bq[1])); return; }
    if (inList) { html.push('</ul>'); inList = false; }
    if (inQuote) { html.push('</blockquote>'); inQuote = false; }
    if (!line.trim()) return;
    html.push('<p>' + inline(line) + '</p>');
    if (inList) { inList = false; }
    if (inQuote) { inQuote = false; }
  });
  if (inList) html.push('</ul>');
  if (inQuote) html.push('</blockquote>');
  return html.join('\n');
}
function inline(s) {
  let r = esc(s);
  r = r.replace(/\*\*([^*]+)\*\*/g, '<b>$1</b>');
  r = r.replace(/`([^`]+)`/g, '<code>$1</code>');
  return r;
}

// Presentation only: the embedded core projection remains the source of state.
const DETAIL_FIELDS = {
  created: 'Created', closed_at: 'Closed', satisfied_at: 'Satisfied at',
  parent_issue: 'Parent issue', pr_url: 'Pull request', responsible: 'Responsible',
  decider: 'Decider', next_evidence: 'Next evidence', closure_note: 'Closure note',
  evidence: 'Evidence', summary: 'Summary', contracts_changed: 'Contracts changed',
  artifacts: 'Artifacts', production: 'Production', deviations: 'Deviations',
  residual: 'Residual', compressed_from: 'Archived record'
};
const LINEAGE_LABELS = new Set(['narrows', 'narrowed-by', 'widens', 'widened-by',
  'supersedes', 'superseded-by', 'completes', 'completed-by']);
function detailValue(value) { return typeof value === 'string' ? value : JSON.stringify(value, null, 2); }
function valueHTML(value) {
  const text = detailValue(value);
  // A URL is a link only when the entire scalar is an HTTP(S) URL.
  if (typeof value === 'string' && /^https?:\/\/[^\s]+$/i.test(value)) {
    return '<a href="' + esc(value) + '" target="_blank" rel="noopener noreferrer">' + esc(value) + '</a>';
  }
  return esc(text);
}
function nodeLink(id) {
  return byId[id] ? '<button class="node-link" data-go="' + esc(id) + '">' + esc(id) + '</button>' : esc(id);
}
function detailBadge(field, label, value, tone = '') {
  return '<span class="detail-badge ' + tone + '" data-field="' + field + '"><span class="badge-label">' + label + '</span> ' + esc(value) + '</span>';
}
function proseClass(text) {
  // Script-sensitive measure, not a language or domain-state inference.
  const letters = String(text).match(/\p{L}/gu) || [];
  const cjk = String(text).match(/[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}]/gu) || [];
  return cjk.length > letters.length / 2 ? 'prose-ja' : 'prose-latin';
}
function focusLabel(id) {
  if (!Object.hasOwn(byId, id)) return 'Show a node neighborhood';
  const plan = pickHop(id);
  return 'Show ' + plan.n + ' hops around ' + id + ' (' + plan.size + ' nodes before filters; up to ' + MODE_THRESHOLD + ' drawn)';
}
function selectNode(id) { closeClusterPanel(); showDetail(id); redraw(); }
function showDetail(id) {
  const n = byId[id];
  if (!n) return;
  const changed = selected !== id;
  selected = id;
  const consumed = new Set(['id', 'type', 'scope', 'title']);
  let badges = detailBadge('type', 'Type', n.type, 'kind-' + n.type) + detailBadge('scope', 'Scope', n.scope);
  if (typeof n.attrs.status === 'string') {
    badges += detailBadge('status', 'Status', n.state || n.status);
    if (n.state && n.status !== n.state) badges += '<span class="status-note">' + esc(n.status) + '</span>';
    consumed.add('status');
  }
  if (n.type === 'criterion') {
    badges += detailBadge('satisfied', 'Acceptance', n.attrs.satisfied === true ? 'Satisfied' : 'Not satisfied', n.attrs.satisfied === true ? 'positive' : '');
    if (typeof n.attrs.satisfied === 'boolean') consumed.add('satisfied');
  }
  if (n.type === 'question' && typeof n.attrs.closed_by === 'string') {
    badges += detailBadge('closed_by', 'Closed by', n.attrs.closed_by);
    consumed.add('closed_by');
  }
  const successors = n.superseded_by || [];
  if (n.type === 'decision') badges += detailBadge('superseded', 'Decision', successors.length ? 'Superseded' : 'Current', successors.length ? 'superseded' : 'positive');
  let declarations = '';
  for (const [key, label] of Object.entries(DETAIL_FIELDS)) {
    if (!Object.prototype.hasOwnProperty.call(n.attrs, key)) continue;
    consumed.add(key);
    const value = n.attrs[key];
    const rendered = ['created','closed_at','satisfied_at'].includes(key) && typeof value === 'string'
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
    if (consumed.has(key)) continue;
    additional += '<div class="additional-attribute" data-key="' + esc(key) + '"><dt>' + esc(key) + '</dt><dd><pre>' + esc(detailValue(value)) + '</pre></dd></div>';
  }
  const relations = relFor(id);
  const group = (title, items) => items.length ? '<section class="detail-relations"><h2>' + title + '</h2>' + items.map(r =>
    '<div class="rel"><span class="relation-chip">' + esc(r.dir + ' ' + r.label) + '</span> ' + nodeLink(r.other) + '</div>').join('') + '</section>' : '';
  const dependencies = DEP.filter(d => d.requirement === id);
  const depHTML = dependencies.length ? '<section id="detailDependencies"><h2>Decision dependencies</h2><table><thead><tr><th>Decision</th><th>Role</th><th>Superseded by</th></tr></thead><tbody>' +
    dependencies.map(d => '<tr><td>' + nodeLink(d.decision) + '</td><td>' + esc(d.role) + '</td><td>' + (d.superseded_by || []).map(nodeLink).join(', ') + '</td></tr>').join('') + '</tbody></table></section>' : '';
  let displayBody = n.body || '';
  const marks = n.body_marks || [];
  marks.forEach(m => {
    if (m.found && m.mark) displayBody = displayBody.split(m.mark).join(m.mark + ' ⟦' + m.label + ': ' + m.source + '⟧');
  });
  const markHTML = marks.length ? '<section id="detailMarks"><h2>Affected passages</h2>' + marks.map(m =>
    '<div class="passage-note" data-found="' + String(m.found) + '"><span class="relation-chip">' + esc(m.label) + '</span> ' + nodeLink(m.source) +
    (m.mark ? '<blockquote>' + esc(m.mark) + '</blockquote><p>' + (m.found ? 'Found in the source body.' : 'Location in body could not be found.') + '</p>' : '<p>No mark identifies the affected passage.</p>') + '</div>').join('') + '</section>' : '';
  document.getElementById('detailBody').innerHTML =
    '<div class="detail-heading"><div class="detail-id">' + esc(n.id) + '</div><h3>' + esc(n.title) + '</h3><button id="detailFocus">' + esc(focusLabel(id)) + '</button><div class="detail-badges">' + badges + '</div>' +
    (successors.length ? '<div class="successors">Superseded by ' + successors.map(nodeLink).join(', ') + '</div>' : '') + '</div>' + scope +
    '<section id="detailContent"><h2>Body</h2><div id="body" class="detail-prose ' + proseClass(n.body || '') + '">' + renderBody(displayBody) + '</div></section>' + markHTML +
    (declarations ? '<section id="detailDeclarations"><h2>Declarations</h2><dl>' + declarations + '</dl></section>' : '') +
    group('Decision lineage', relations.filter(r => LINEAGE_LABELS.has(r.label))) +
    group('Relationships', relations.filter(r => !LINEAGE_LABELS.has(r.label))) + depHTML +
    (additional ? '<section id="detailAdditional"><h2>Additional attributes</h2><dl>' + additional + '</dl></section>' : '');
  const detail = document.getElementById('detail');
  detail.classList.add('on');
  if (changed) detail.scrollTop = 0;
}
function hideDetail() {
  document.getElementById('detail').classList.remove('on');
  selected = null;
}
function closeDetail() { hideDetail(); redraw(); }
document.getElementById('closeDetail').addEventListener('click', closeDetail);
document.getElementById('detail').addEventListener('click', (e) => {
  const t = e.target.closest('[data-go]');
  if (t) selectNode(t.dataset.go);
  if (e.target.closest('#detailFocus') && selected) startHop(selected);
});

