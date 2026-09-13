import { EDGES } from '../data';
import { element } from '../dom';
import { token } from '../tokens';
// ---------- SVG rendering ----------
export const svg = element('svg');
export const NS = 'http:' + '//www.w3.org/2000/svg';
export const root = (document.createElementNS(NS, 'g') as SVGElement);
export function initSvg() { svg.appendChild(root); }
let defs = null;
export function makeDefs() {
    defs = (document.createElementNS(NS, 'defs') as SVGElement);
    svg.insertBefore(defs, root);
    // arrowheads per edge label
    const labels = [...new Set(EDGES.map(e => e.label))];
    labels.forEach(l => {
        const mk = (document.createElementNS(NS, 'marker') as SVGElement);
        mk.setAttribute('id', String('arr-' + l.replace(/\W/g, '_')));
        mk.setAttribute('viewBox', '0 -4 8 8');
        mk.setAttribute('refX', '9');
        mk.setAttribute('refY', '0');
        mk.setAttribute('markerWidth', '7');
        mk.setAttribute('markerHeight', '7');
        mk.setAttribute('orient', 'auto');
        const p = (document.createElementNS(NS, 'path') as SVGElement);
        p.setAttribute('d', 'M0,-4L8,0L0,4');
        p.setAttribute('fill', String(edgeColor(l)));
        mk.appendChild(p);
        defs.appendChild(mk);
    });
    // aggregated-edge arrowhead (overview mode)
    const agg = (document.createElementNS(NS, 'marker') as SVGElement);
    agg.setAttribute('id', 'arr-_agg');
    agg.setAttribute('viewBox', '0 -4 8 8');
    agg.setAttribute('refX', '9');
    agg.setAttribute('refY', '0');
    agg.setAttribute('markerWidth', '7');
    agg.setAttribute('markerHeight', '7');
    agg.setAttribute('orient', 'auto');
    const pa = (document.createElementNS(NS, 'path') as SVGElement);
    pa.setAttribute('d', 'M0,-4L8,0L0,4');
    pa.setAttribute('fill', String(token('--color-888')));
    agg.appendChild(pa);
    defs.appendChild(agg);
}
export function edgeColor(label) { const map = { closes: token('--color-a03b3b'), narrows: token('--decision'), widens: token('--color-3f8f6b'), supersedes: token('--color-8a2b2b'), completes: token('--ok'), targets: token('--criterion'), 'spawned-by': token('--gate'), 'filed-as': token('--color-7d6c5f'), 'depends-on': token('--need'), 'relies-on': token('--color-5a5a6e'), raised: token('--color-a8608a'), 'measured-by': token('--color-556b7a') }; return map[label] || token('--color-666'); }
export function shapeOf(type) {
    const s = {
        need: 'M0,-7 C4,-7 7,-4 7,0 C7,4 4,7 0,7 C-4,7 -7,4 -7,0 C-7,-4 -4,-7 0,-7 Z',
        question: 'M0,-9 L2.5,-2 L9,0 L2.5,2 L0,9 L-2.5,2 L-9,0 L-2.5,-2 Z',
        decision: 'M-7,-7 L7,-7 L7,7 L-7,7 Z',
        requirement: 'M-7,-4 L0,-7 L7,-4 L7,4 L0,7 L-7,4 Z',
        criterion: 'M0,-7 L7,5 L-7,5 Z',
        gate: 'M0,-8 L7.6,-2.5 L4.7,6.5 L-4.7,6.5 L-7.6,-2.5 Z'
    };
    return s[type] || s.decision;
}
