import { esc } from '../components';
import { D, EDGES } from '../data';
import { element } from '../dom';
import { NS, edgeColor } from '../graph/svg';
import { KIND_COLORS, token } from '../tokens';
// ---------- progress charts ----------
export function bars(elId, data, color) {
    const el = element(elId);
    const W = 400, H = 120, pad = 6;
    el.innerHTML = '';
    el.setAttribute('viewBox', '0 0 400 120');
    const max = Math.max(1, ...data.map(d => d.value));
    data.forEach((d, i) => {
        const barW = (W - pad * 2) / data.length - 2;
        const h = (d.value / max) * (H - pad * 2);
        const r = (document.createElementNS(NS, 'rect') as SVGElement);
        r.setAttribute('x', String(pad + i * (barW + 2)));
        r.setAttribute('y', String(H - pad - h));
        r.setAttribute('width', String(barW));
        r.setAttribute('height', String(h));
        r.setAttribute('fill', String(color));
        const txt = (document.createElementNS(NS, 'text') as SVGElement);
        txt.setAttribute('x', String(pad + i * (barW + 2) + barW / 2));
        txt.setAttribute('y', String(H - pad - h - 3));
        txt.setAttribute('text-anchor', 'middle');
        txt.setAttribute('font-size', token('--font-7px'));
        txt.textContent = d.label;
        el.appendChild(r);
        el.appendChild(txt);
    });
}
// AC timeline
export function initProgress() {
    const acData = D.criteria_timeline || [{ label: 'today', value: D.criteria.satisfied || 0 }];
    bars('acChart', acData, token('--criterion'));
    element('acChartNote').textContent = (D.criteria.satisfied + ' / ' + D.criteria.total + ' acceptance criteria satisfied');
    // arrival
    if (D.arrival && D.arrival.available) {
        const qd = [{ label: 'prev', value: Math.round((D.arrival.previous_per_day || 0) * 10) / 10 }, { label: 'now', value: Math.round((D.arrival.current_per_day || 0) * 10) / 10 }];
        bars('qChart', qd, token('--gate'));
        element('arrivalNote').textContent = 'new questions/day: ' + D.arrival.current_per_day.toFixed(2) + ' now, ' + D.arrival.previous_per_day.toFixed(2) + ' before; decay ' + (D.arrival.decay_fraction == null ? 'n/a' : (D.arrival.decay_fraction * 100).toFixed(0) + '%');
    }
    else {
        element('qChart').innerHTML = '';
        element('arrivalNote').textContent = 'Question arrival is unavailable: no git history for this ledger.';
    }
    // ---------- legend ----------
    (function buildLegend() {
        const box = element('legendBox');
        let html = '<strong>Legend</strong>';
        Object.entries(KIND_COLORS).forEach(([k, c]) => {
            html += '<span class="legend-item"><span class="swatch ' + (k === 'criterion' ? 'shp-criterion' : '') + '" style="background:' + c + '"></span>' + k + '</span>';
        });

        const labels = [...new Set(EDGES.map(e => e.label))];
        labels.forEach(l => {
            html += '<div class="edge-row"><span class="eline" style="border-color:' + edgeColor(l) + '"></span>' + esc(l) + '</div>';
        });
        html += '<div class="edge-row"><span class="swatch shp-need" style="background:transparent;border:2px solid var(--color-ffb000)"></span>search hit</div>';
        html += '<div class="edge-row"><span class="swatch shp-need" style="background:var(--hl);border:1px solid var(--color-222)"></span>selected</div>';

        box.innerHTML = html;
    })();
}
