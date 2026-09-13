// ---------- progress charts ----------
function bars(elId, data, color) {
  const el = document.getElementById(elId);
  const W = 400, H = 120, pad = 6;
  el.innerHTML = '';
  el.setAttribute('viewBox', '0 0 400 120');
  const max = Math.max(1, ...data.map(d => d.value));
  data.forEach((d, i) => {
    const barW = (W - pad*2) / data.length - 2;
    const h = (d.value / max) * (H - pad*2);
    const r = document.createElementNS(NS, 'rect');
    r.setAttribute('x', pad + i*(barW+2)); r.setAttribute('y', H - pad - h);
    r.setAttribute('width', barW); r.setAttribute('height', h);
    r.setAttribute('fill', color);
    const txt = document.createElementNS(NS, 'text');
    txt.setAttribute('x', pad + i*(barW+2) + barW/2); txt.setAttribute('y', H - pad - h - 3);
    txt.setAttribute('text-anchor', 'middle'); txt.setAttribute('font-size', '7');
    txt.textContent = d.label;
    el.appendChild(r); el.appendChild(txt);
  });
}
// AC timeline
const acData = D.criteria_timeline || [{label:'today', value:D.criteria.satisfied || 0}];
bars('acChart', acData, '#c96f6f');
document.getElementById('acChartNote').textContent = (D.criteria.satisfied + ' / ' + D.criteria.total + ' acceptance criteria satisfied'); 
// arrival
if (D.arrival && D.arrival.available) {
  const qd = [{label:'prev', value: Math.round((D.arrival.previous_per_day||0)*10)/10}, {label:'now', value: Math.round((D.arrival.current_per_day||0)*10)/10}];
  bars('qChart', qd, '#4f9aa8');
  document.getElementById('arrivalNote').textContent = 'new questions/day: ' + D.arrival.current_per_day.toFixed(2) + ' now, ' + D.arrival.previous_per_day.toFixed(2) + ' before; decay ' + (D.arrival.decay_fraction == null ? 'n/a' : (D.arrival.decay_fraction*100).toFixed(0) + '%');
} else {
  document.getElementById('qChart').innerHTML = '';
  document.getElementById('arrivalNote').textContent = 'Question arrival is unavailable: no git history for this ledger.';
}

// ---------- legend ----------
(function buildLegend(){
  const box = document.getElementById('legendBox');
  let html = '<div style="font-weight:600;margin-bottom:4px">Legend</div>';
  html += '<div style="display:grid;grid-template-columns:1fr 1fr;gap:2px 8px;margin-bottom:6px">';
  Object.entries(KIND_COLORS).forEach(([k,c]) => {
    html += '<span style="display:flex;align-items:center;gap:4px"><span class="swatch ' + (k==='criterion'?'shp-criterion':'') + '" style="background:' + c + '"></span>' + k + '</span>';
  });
  html += '</div><div>';
  const labels = [...new Set(EDGES.map(e=>e.label))];
  labels.forEach(l => {
    html += '<div class="edge-row"><span class="eline" style="border-color:' + edgeColor(l) + '"></span>' + esc(l) + '</div>';
  });
  html += '<div class="edge-row"><span class="swatch shp-need" style="background:transparent;border:2px solid #ffb000"></span>search hit</div>';
  html += '<div class="edge-row"><span class="swatch shp-need" style="background:#ffd766;border:1px solid #222"></span>selected</div>';
  html += '</div>';
  box.innerHTML = html;
})();

