// header meta
document.getElementById('metaGen').textContent = 'Generated: ' + D.generated_at;
document.getElementById('metaScope').textContent = 'Scopes: ' + (D.scope ? D.scope : 'all (' + (D.scopes||[]).join(', ') + ')');
document.getElementById('metaNodes').textContent = 'Nodes: ' + D.node_count;
document.getElementById('integrityNote').textContent = D.integrity_note;

// ---------- overview numbers ----------
const lintArr = D.lint || [];
const lintErr = lintArr.filter(d => d.severity === 'error').length;
const lintWarn = lintArr.filter(d => d.severity === 'warn').length;
const openQs = (D.open_questions || []);
const waitRefs = openQs.reduce((a, q) => a + (q.referencing || []).length, 0);
document.getElementById('acSatisfied').textContent = D.criteria.satisfied + ' / ' + D.criteria.total;
document.getElementById('acTotal').textContent = D.criteria.satisfied;
document.getElementById('openQ').textContent = openQs.length;
document.getElementById('waitRefs').textContent = waitRefs;
document.getElementById('nextCount').textContent = (D.next || []).length;
document.getElementById('lintErr').textContent = lintErr;
document.getElementById('lintWarn').textContent = lintWarn;

// state bars
const states = D.states || {};
const maxState = Math.max(1, ...Object.values(states));
const stateColors = ['#8a6fb2','#7a9e7e','#4f9aa8','#4f7fb8','#d9a441','#c97b4f','#c96f6f','#a8608a','#7d6c5f','#556b7a','#3f8f6b'];
let i = 0;
const stateBar = document.getElementById('stateBars');
Object.entries(states).forEach(([st, cnt]) => {
  const w = Math.round(cnt / maxState * 100);
  const d = document.createElement('div');
  d.className = 'bar';
  d.style.setProperty('--w', w + '%');
  d.style.setProperty('--bar', stateColors[i++ % stateColors.length]);
  d.title = st;
  d.innerHTML = esc(st) + ' <em>' + cnt + '</em>';
  d.addEventListener('click', () => { setStateFilter(st); gotoTab('filters'); });
  stateBar.appendChild(d);
});

