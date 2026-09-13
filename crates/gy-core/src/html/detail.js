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

// Source HTML is always text. Only renderer-owned tags enter the detail DOM.
function inline(source) {
  const s = String(source), out = [];
  for (let i = 0; i < s.length;) {
    if (s[i] === '\\' && s[i+1] && /[^\w\s]/.test(s[i+1])) {
      out.push(esc(s[i+1])); i += 2; continue;
    }
    if (s[i] === '<' && /^<(?:\/?[A-Za-z]|!--)/.test(s.slice(i))) {
      let end = i+1, quote = '';
      for (; end < s.length; end++) {
        if (quote) { if (s[end] === quote) quote = ''; }
        else if (s[end] === '\"' || s[end] === "'") quote = s[end];
        else if (s[end] === '>') break;
      }
      if (end < s.length) { out.push(esc(s.slice(i,end+1))); i = end+1; continue; }
    }
    if (s[i] === '`') {
      const ticks = s.slice(i).match(/^`+/)[0], end = s.indexOf(ticks, i+ticks.length);
      if (end >= 0) {
        let code = s.slice(i+ticks.length, end).replace(/\n/g, ' ');
        if (/^ .* $/.test(code) && code.trim()) code = code.slice(1,-1);
        out.push('<code>' + esc(code) + '</code>'); i = end+ticks.length; continue;
      }
    }
    if (s[i] === '[') {
      const middle = s.indexOf('](', i+1);
      if (middle >= 0) {
        let end = middle+2, depth = 1;
        for (; end < s.length; end++) {
          if (s[end] === '\\') { end++; continue; }
          if (s[end] === '(') depth++;
          if (s[end] === ')' && --depth === 0) break;
        }
        if (depth === 0) {
          const destination = s.slice(middle+2, end).match(/^(?:<([^>]*)>|(\S+?))(?:\s+"([^"]*)")?$/);
          if (destination) {
            const href = (destination[1] ?? destination[2]).replace(/\\([()])/g, '$1');
            let safe = false;
            try { safe = ['http:', 'https:', 'mailto:', 'file:'].includes(new URL(href, location.href).protocol) && !/[\u0000-\u0020]/.test(href); } catch (_) {}
            const label = inline(s.slice(i+1, middle));
            out.push(safe ? '<a href="' + esc(href) + '" rel="noopener noreferrer"' + (destination[3] ? ' title="' + esc(destination[3]) + '"' : '') + '>' + label + '</a>' : label + ' (' + esc(href) + ')');
            i = end+1; continue;
          }
        }
      }
    }
    let matched = false;
    for (const [mark, open, close] of [['***','<strong><em>','</em></strong>'],['___','<strong><em>','</em></strong>'],['**','<strong>','</strong>'],['__','<strong>','</strong>'],['~~','<del>','</del>'],['*','<em>','</em>'],['_','<em>','</em>']]) {
      if (!s.startsWith(mark, i) || /\s/.test(s[i+mark.length] || ' ') || (mark[0] === '_' && /\w/.test(s[i-1] || ''))) continue;
      let end = s.indexOf(mark, i+mark.length);
      while (end >= 0 && s[end-1] === '\\') end = s.indexOf(mark, end+mark.length);
      if (end > i+mark.length && !/\s/.test(s[end-1])) {
        out.push(open + inline(s.slice(i+mark.length,end)) + close); i = end+mark.length; matched = true; break;
      }
    }
    if (!matched) { out.push(esc(s[i])); i++; }
  }
  return out.join('');
}
function tableCells(line) {
  const cells = [], text = line.trim();
  let cell = '', ticks = 0;
  for (let i = 0; i < text.length; i++) {
    if (text[i] === '\\' && text[i+1] === '|') { cell += '\\|'; i++; continue; }
    if (text[i] === '`') {
      const run = text.slice(i).match(/^`+/)[0];
      ticks = ticks === run.length ? 0 : (ticks || run.length);
      cell += run; i += run.length-1; continue;
    }
    if (text[i] === '|' && !ticks) { cells.push(cell.trim()); cell = ''; }
    else cell += text[i];
  }
  cells.push(cell.trim());
  if (text.startsWith('|')) cells.shift();
  if (text.endsWith('|') && cells[cells.length-1] === '') cells.pop();
  return cells;
}
function renderBody(md) {
  const lines = String(md || '').replace(/\r\n?/g, '\n').split('\n');
  const fence = line => line.match(/^ {0,3}(`{3,}|~{3,})(.*)$/);
  const indentLine = line => line.replace(/^[ \t]*/, prefix => { let width = 0; for (const c of prefix) width += c === '\t' ? 4-width%4 : 1; return ' '.repeat(width); });
  const item = line => indentLine(line).match(/^( *)([-+*]|\d+[.)]) +(.*)$/);
  const heading = line => line.match(/^ {0,3}(#{1,6})\s+(.*)$/);
  function blocks(lines) {
    const html = [];
    let i = 0;
    const isTable = at => at+1 < lines.length && lines[at].includes('|') &&
      tableCells(lines[at]).length > 0 && tableCells(lines[at+1]).length === tableCells(lines[at]).length &&
      tableCells(lines[at+1]).every(c => /^:?-+:?$/.test(c));
    while (i < lines.length) {
      const line = lines[i];
      if (!line.trim()) { i++; continue; }
      const f = fence(line);
      if (f) {
        const code = [], close = new RegExp('^ {0,3}' + f[1][0] + '{' + f[1].length + ',}\\s*$');
        i++;
        while (i < lines.length && !close.test(lines[i])) code.push(lines[i++]);
        if (i < lines.length) i++;
        const language = f[2].trim().split(/\s/)[0];
        html.push('<pre><code' + (language ? ' class="language-' + esc(language) + '"' : '') + '>' + esc(code.join('\n') + (code.length ? '\n' : '')) + '</code></pre>');
        continue;
      }
      if (/^ {0,3}<(?:\/?[A-Za-z]|!--)/.test(line)) {
        const literal = [];
        while (i < lines.length && lines[i].trim()) literal.push(lines[i++]);
        html.push('<p>' + esc(literal.join('\n')) + '</p>'); continue;
      }
      const h = heading(line);
      if (h) { html.push('<h4>' + inline(h[2].replace(/\s+#+\s*$/, '')) + '</h4>'); i++; continue; }
      if (isTable(i)) {
        const headers = tableCells(line), align = tableCells(lines[i+1]).map(c => c.endsWith(':') ? (c.startsWith(':') ? 'center' : 'right') : 'left');
        const row = (cells, tag) => '<tr>' + headers.map((_,n) => '<' + tag + ' style="text-align:' + align[n] + '">' + inline(cells[n] || '') + '</' + tag + '>').join('') + '</tr>';
        html.push('<table><thead>' + row(headers,'th') + '</thead><tbody>'); i += 2;
        while (i < lines.length && lines[i].trim() && lines[i].includes('|')) html.push(row(tableCells(lines[i++]),'td'));
        html.push('</tbody></table>'); continue;
      }
      const first = item(line);
      if (first) {
        const indent = first[1].length, ordered = /^\d/.test(first[2]), tag = ordered ? 'ol' : 'ul';
        html.push('<' + tag + (ordered ? ' start="' + parseInt(first[2],10) + '"' : '') + '>');
        while (i < lines.length) {
          const entry = item(lines[i]);
          if (!entry || entry[1].length !== indent || /^\d/.test(entry[2]) !== ordered) break;
          const contentIndent = indentLine(lines[i]).length-entry[3].length, content = [entry[3]]; i++;
          while (i < lines.length) {
            const next = indentLine(lines[i]), spaces = next.match(/^ */)[0].length;
            if (!next.trim()) { content.push(''); i++; continue; }
            if (spaces > indent) { content.push(next.slice(Math.min(contentIndent,spaces))); i++; continue; }
            if (item(next) || heading(next) || fence(next) || content[content.length-1] === '') break;
            content.push(next); i++;
          }
          html.push('<li>' + blocks(content) + '</li>');
        }
        html.push('</' + tag + '>'); continue;
      }
      if (/^ {0,3}>/.test(line)) {
        const quoted = [];
        while (i < lines.length && /^ {0,3}>/.test(lines[i])) quoted.push(lines[i++].replace(/^ {0,3}> ?/,''));
        html.push('<blockquote>' + blocks(quoted) + '</blockquote>'); continue;
      }
      const paragraph = [line]; i++;
      while (i < lines.length && lines[i].trim() && !fence(lines[i]) && !heading(lines[i]) && !item(lines[i]) && !/^ {0,3}>/.test(lines[i]) && !isTable(i)) paragraph.push(lines[i++]);
      html.push('<p>' + inline(paragraph.join('\n')) + '</p>');
    }
    return html.join('\n');
  }
  return blocks(lines);
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
function focusPlan(id) {
  const radius = Number(document.getElementById('hopRadius').value);
  return radius ? {n:radius, size:reachable(id, radius).size} : pickHop(id);
}
function focusLabel(id) {
  if (!Object.hasOwn(byId, id)) return 'Show a node neighborhood';
  const plan = focusPlan(id);
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

