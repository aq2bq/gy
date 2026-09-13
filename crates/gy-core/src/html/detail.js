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
  const blocks = [];
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

function attrRows(n) {
  const rows = [];
  for (const [k, v] of Object.entries(n.attrs)) {
    const str = typeof v === 'string' ? v : JSON.stringify(v, null, 1);
    rows.push('<tr><th>' + esc(k) + '</th><td><pre>' + esc(str) + '</pre></td></tr>');
  }
  return rows.join('');
}

function showDetail(id) {
  const n = byId[id];
  if (!n) return;
  selected = id;
  const el = document.getElementById('detailBody');
  const rel = relFor(id);
  const rx = rel.map(r => '<div class="rel">' + esc(r.other) + ' <span class="lbl">' + esc(r.label) + ' ' + r.dir + '</span></div>').join('');
  let depRows = '';
  const myDeps = DEP.filter(d => d.requirement === id);
  if (myDeps.length) {
    depRows = '<h2 style="position:inherit">Decision dependencies</h2><table><tr><th>decision</th><th>role</th><th>superseded by</th></tr>' +
      myDeps.map(d => '<tr><td>' + esc(d.decision) + '</td><td>' + esc(d.role) + '</td><td>' + esc((d.superseded_by||[]).join(', ')) + '</td></tr>').join('') +
      '</table>';
  }
  const superseded = dSuperseded[id];
  const supNote = superseded ? '<div class="rel"><span class="lbl">This decision is superseded by:</span> ' + superseded.map(esc).join(', ') + '</div>' : '';
  const gitHubId = (n.attrs['parent_issue'] !== undefined) ? ' · parent #' + esc(String(n.attrs['parent_issue'])) : '';
  el.innerHTML =
    '<h3>' + esc(n.id) + '</h3>'
    + '<div class="sub">' + esc(n.type) + ' · ' + esc(n.scope) + (n.status ? ' · ' + esc(n.status) : '') + ' · created ' + esc(n.created || '') + '</div>'
    + '<div style="margin-bottom:8px"><b>' + esc(n.title) + '</b></div>'
    + '<h2 style="position:inherit">Attributes</h2><table>' + attrRows(n) + '</table>'
    + '<h2 style="position:inherit">Relationships</h2>' + (rx || '<div class="small">none</div>')
    + supNote + depRows
    + '<h2 style="position:inherit">Body</h2><div id="body">' + renderBody(n.body || '') + '</div>';
  document.getElementById('detail').classList.add('on');
}
function hideDetail() {
  document.getElementById('detail').classList.remove('on');
  selected = null;
}
function closeDetail() { hideDetail(); redraw(); }
document.getElementById('closeDetail').addEventListener('click', closeDetail);
document.getElementById('detail').addEventListener('click', (e) => {
  const t = e.target.closest('[data-go]');
  if (t) { showDetail(t.dataset.go); }
});

// superseded marked per node in payload; also keep the map for the detail panel
const dSuperseded = {};
EDGES.forEach(e => {
  if (e.label === 'supersedes') { (dSuperseded[e.target] = dSuperseded[e.target] || []).push(e.source); }
});

