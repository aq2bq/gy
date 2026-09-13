// ---------- blockers (jam) ----------
function jamList(ulEl, items, tag) {
  const ul = document.getElementById(ulEl);
  ul.innerHTML = '';
  items.forEach(it => {
    const li = document.createElement('li');
    const a = document.createElement('a');
    a.dataset.go = it.id;
    a.innerHTML = '<span class="id">' + esc(it.id) + '</span> <span class="lbl">' + esc(it.label) + '</span>';
    li.appendChild(a);
    ul.appendChild(li);
  });
  if (!items.length) { ul.innerHTML = '<li class="small">none</li>'; }
}
// open questions
jamList('jamQ', (D.open_questions || []).flatMap(q => [
  {id: q.id, label: (q.title || '') + (q.referencing && q.referencing.length ? ' — referenced by ' + q.referencing.join(', ') : '')}
]), 'q');
// next needs
jamList('jamNext', (D.next || []), 'next');
// missing
jamList('jamMissing', (D.missing || []).map(m => ({id: m.id, label: 'next_evidence ' + (m.next_evidence ? 'set' : 'empty') + '; responsible ' + (m.responsible ? 'set' : 'empty')})), 'missing');
// dangling
jamList('jamDangling', (D.dangling || []).map(d => ({id: d.source, label: 'references missing node ' + d.target})), 'dangling');
// lint
const lintUl = document.getElementById('jamLint');
lintUl.innerHTML = '';
lintArr.forEach(d => {
  const li = document.createElement('li');
  const a = document.createElement('a');
  a.dataset.go = d.id;
  a.innerHTML = '<span class="lint-sev" style="color:' + (d.severity === 'error' ? 'var(--err)' : 'var(--warn)') + '">' + esc(d.severity) + '</span> ' +
    '<span class="id">' + esc(d.rule) + '</span> <span class="lbl">' + esc(d.id) + ' — ' + esc(d.message) + '</span>';
  li.appendChild(a);
  lintUl.appendChild(li);
});
if (!lintArr.length) lintUl.innerHTML = '<li class="small">no findings</li>';
document.getElementById('detail').addEventListener('click', (e) => {
  const t = e.target.closest('[data-go]');
  if (t) { showDetail(t.dataset.go); }
});

