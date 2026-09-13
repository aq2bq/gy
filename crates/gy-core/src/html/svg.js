// ---------- SVG rendering ----------
const svg = document.getElementById('svg');
const NS = 'http:' + '//www.w3.org/2000/svg';
const root = document.createElementNS(NS, 'g');
svg.appendChild(root);
let defs = null;

function makeDefs() {
  defs = document.createElementNS(NS, 'defs');
  svg.insertBefore(defs, root);
  // arrowheads per edge label
  const labels = [...new Set(EDGES.map(e => e.label))];
  labels.forEach(l => {
    const mk = document.createElementNS(NS, 'marker');
    mk.setAttribute('id', 'arr-' + l.replace(/\W/g, '_'));
    mk.setAttribute('viewBox', '0 -4 8 8');
    mk.setAttribute('refX', '9'); mk.setAttribute('refY', '0');
    mk.setAttribute('markerWidth', '7'); mk.setAttribute('markerHeight', '7');
    mk.setAttribute('orient', 'auto');
    const p = document.createElementNS(NS, 'path');
    p.setAttribute('d', 'M0,-4L8,0L0,4');
    p.setAttribute('fill', edgeColor(l));
    mk.appendChild(p);
    defs.appendChild(mk);
  });
  // aggregated-edge arrowhead (overview mode)
  const agg = document.createElementNS(NS, 'marker');
  agg.setAttribute('id', 'arr-_agg');
  agg.setAttribute('viewBox', '0 -4 8 8');
  agg.setAttribute('refX', '9'); agg.setAttribute('refY', '0');
  agg.setAttribute('markerWidth', '7'); agg.setAttribute('markerHeight', '7');
  agg.setAttribute('orient', 'auto');
  const pa = document.createElementNS(NS, 'path');
  pa.setAttribute('d', 'M0,-4L8,0L0,4');
  pa.setAttribute('fill', '#888');
  agg.appendChild(pa);
  defs.appendChild(agg);
}
function edgeColor(label){ const map = {closes:'#a03b3b',narrows:'#4f7fb8',widens:'#3f8f6b',supersedes:'#8a2b2b',completes:'#2f6f45',targets:'#c96f6f','spawned-by':'#4f9aa8','filed-as':'#7d6c5f','depends-on':'#7a9e7e','relies-on':'#5a5a6e',raised:'#a8608a','measured-by':'#556b7a'}; return map[label] || '#666'; }

function shapeOf(type){
  const s = {
    need:    'M0,-7 C4,-7 7,-4 7,0 C7,4 4,7 0,7 C-4,7 -7,4 -7,0 C-7,-4 -4,-7 0,-7 Z',
    question:'M0,-9 L2.5,-2 L9,0 L2.5,2 L0,9 L-2.5,2 L-9,0 L-2.5,-2 Z',
    decision:'M-7,-7 L7,-7 L7,7 L-7,7 Z',
    requirement:'M-7,-4 L0,-7 L7,-4 L7,4 L0,7 L-7,4 Z',
    criterion:'M0,-7 L7,5 L-7,5 Z',
    gate:'M0,-8 L7.6,-2.5 L4.7,6.5 L-4.7,6.5 L-7.6,-2.5 Z'
  };
  return s[type] || s.decision;
}

