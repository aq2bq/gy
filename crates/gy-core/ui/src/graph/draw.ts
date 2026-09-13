import { clusterWidth, nodeLabel, LINE_HEIGHT } from './labels';
import { EDGES, byId } from '../data';
import { selectNode } from '../detail/index';
import { element } from '../dom';
import { renderNavigation } from '../navigation';
import { EDGE_LABEL_MAX, MODE_THRESHOLD, currentFocus, dragMoved, forceLayout, genealogyLayout, genealogyMode, lastFitKey, lastViewport, lod, positions, scale, searchHits, selected, setLastFitKey, setLastViewport, setTranslate, translate, viewSource } from '../state';
import { KIND_COLORS, token } from '../tokens';
import { openClusterList, revealOmitted } from './clusters';
import { renderList } from '../list';
import { ensureForce } from './layout';
import { focusedSelection, visibleEdges } from './selection';
import { NS, edgeColor, root, shapeOf, svg } from './svg';
import { applyTransform, fitTransform } from './viewport';
// ---------- draw dispatch ----------
// Compute overview cluster centers so the grid matches the viewport aspect.
export function overviewGrid(ids) {
    const cluster = {};
    ids.forEach(id => { const n = byId[id]; const g = n.scope + '\u0000' + n.type; (cluster[g] = cluster[g] || []).push(id); });
    const gs = Object.keys(cluster);
    const cells = {};
    if (!gs.length)
        return { cluster, cells };
    const cellW = Math.max(...gs.map(g=>clusterWidth(g,cluster[g].length)))+24, cellH = 72;
    const vp = svg.getBoundingClientRect();
    const n = gs.length;
    // Choose columns from measured cluster widths and available screen space.
    let cols = n, rows = 1, best = 0;
    for (let c = 1; c <= n; c++) {
        const r = Math.ceil(n / c);
        const score = Math.min(vp.width/(c*cellW),vp.height/(r*cellH));
        if (score > best) {
            best = score;
            cols = c;
            rows = r;
        }
    }
    const gridW = cols * cellW, gridH = rows * cellH;
    gs.forEach((g, i) => {
        const c = i % cols, r = Math.floor(i / cols);
        cells[g] = { bx: c * cellW + cellW / 2 - gridW / 2, by: r * cellH + cellH / 2 - gridH / 2 };
    });
    return { cluster, cells };
}
export function dotPath(sx, sy, tx, ty) {
    const dx = tx - sx, dy = ty - sy, d = Math.max(1, Math.hypot(dx, dy));
    return { x1: sx, y1: sy, x2: tx - dx / d * 16, y2: ty - dy / d * 16 };
}
// Refit automatically whenever the visible set changes (overview <-> neighborhood,
// search/filter, genealogy entry), so the new content is not left outside the viewport.
export function draw() {
    const { ids, omitted } = focusedSelection();
    renderList();
    renderNavigation(ids);
    const viewport = svg.getBoundingClientRect();
    const focus = currentFocus();
    const key = JSON.stringify([genealogyMode, focus.id, focus.radius, [...ids].sort()]);
    if (key !== lastFitKey) {
        setLastFitKey(key);
        fitTransform();
    }
    else if (lastViewport && (viewport.width !== lastViewport.width || viewport.height !== lastViewport.height)) {
        if (viewSource === 'fit')
            fitTransform();
        else {
            // Preserve the world point at the viewport center and the user's scale.
            setTranslate({ ...translate, x: translate.x + ((viewport.width - lastViewport.width) / 2) });
            setTranslate({ ...translate, y: translate.y + ((viewport.height - lastViewport.height) / 2) });
            applyTransform();
        }
    }
    setLastViewport({ width: viewport.width, height: viewport.height });
    const showingAll = ids.length <= MODE_THRESHOLD || genealogyMode;
    if (!genealogyMode && showingAll && ids.length > 0)
        ensureForce(ids);
    const inPositions = genealogyMode ? genealogyLayout : (showingAll ? forceLayout : positions);
    // clear
    root.innerHTML = '';
    const culling = element('culling');
    const setMeta = (drawn, visCount, extra) => {
        const overview = !showingAll;
        element('viewCounts').textContent = overview
            ? 'Graph: ' + drawn + (drawn === 1 ? ' cluster' : ' clusters')
            : 'Graph: ' + drawn + ' shown';
        culling.replaceChildren();
        const off = overview ? 0 : Number(visCount) - drawn;
        const hidden = off + omitted.length;
        if (!hidden) culling.textContent = '0 hidden';
        if (off) culling.appendChild(document.createTextNode(off + ' off-screen at readable zoom'));
        if (omitted.length) {
            if (off) culling.appendChild(document.createTextNode(' · '));
            const button = document.createElement('button');
            button.id = 'showOmitted';
            button.textContent = omitted.length + ' nodes omitted';
            button.addEventListener('click', () => revealOmitted(omitted));
            culling.appendChild(button);
        }
    };
    if (genealogyMode) {
        drawEdgesLayer(inPositions, ids.filter(id => inPositions[id]), visibleEdges(ids).filter(e => (e.label === 'narrows' || e.label === 'supersedes' || e.label === 'completes' || e.label === 'widens') && inPositions[e.source] && inPositions[e.target]));
        const n = drawNodeLayer(inPositions, ids);
        const off = offscreenCount(inPositions, ids);
        setMeta(n - off, n, off > 0 ? ('Showing ' + (n - off) + ' of ' + n + ' nodes; ' + off + ' off-screen at readable zoom. Zoom out or pan to reach them.') : '');
        return;
    }
    if (!showingAll) {
        // Overview: clusters + aggregated edges, no individual nodes.  ------
        const { cluster, cells } = overviewGrid(ids);
        const cg = Object.keys(cluster);
        // Aggregated edges between clusters (H19)
        const pair = {};
        EDGES.forEach(e => {
            if (!ids.includes(e.source) || !ids.includes(e.target))
                return;
            const gs = byId[e.source].scope + '\u0000' + byId[e.source].type;
            const gt = byId[e.target].scope + '\u0000' + byId[e.target].type;
            if (gs === gt)
                return; // kept as internal count (H26)
            const k = gs + '|' + gt;
            (pair[k] = pair[k] || []).push(e.label);
        });
        const edgeLayer = (document.createElementNS(NS, 'g') as SVGElement);
        const pairKeys = Object.keys(pair);
        // Fan inter-cluster edges out with quadratic curves so that edges sharing a
        // lane separate and every label can be traced back to its two clusters (H19).
        pairKeys.forEach((k, i) => {
            const labels = pair[k];
            const [gs, gt] = k.split('|');
            const a = cells[gs] || { bx: 0, by: 0 };
            const b = cells[gt] || { bx: 0, by: 0 };
            const sx = a.bx, sy = a.by, tx = b.bx, ty = b.by;
            const dx = tx - sx, dy = ty - sy, len = Math.max(1, Math.hypot(dx, dy));
            // perpendicular unit vector; alternate sides by pair index
            const ux = -dy / len, uy = dx / len;
            const side = (i % 2 === 0 ? 1 : -1);
            const dist = 90 + Math.floor(i / 2) * 28;
            const mx = (sx + tx) / 2 + ux * dist * side;
            const my = (sy + ty) / 2 + uy * dist * side;
            const endX = tx - dx / len * 16, endY = ty - dy / len * 16;
            const path = (document.createElementNS(NS, 'path') as SVGElement);
            path.setAttribute('d', String('M ' + sx + ' ' + sy + ' Q ' + mx + ' ' + my + ' ' + endX + ' ' + endY));
            path.setAttribute('fill', 'none');
            path.setAttribute('stroke', String(token('--color-888')));
            path.setAttribute('stroke-width', '1.4');
            path.setAttribute('marker-end', 'url(#arr-_agg)');
            path.setAttribute('opacity', '0.75');
            path.setAttribute('data-label', String([...new Set(labels)].join(',')));
            path.setAttribute('data-s', String(gs));
            path.setAttribute('data-t', String(gt));
            edgeLayer.appendChild(path);
            const t = (document.createElementNS(NS, 'text') as SVGTextElement);
            t.setAttribute('x', String(mx + ux * 14 * side));
            t.setAttribute('y', String(my + uy * 14 * side - 4));
            t.setAttribute('text-anchor', 'middle');
            t.setAttribute('class', 'elabel');
            t.setAttribute('data-count', String(labels.length));
            t.setAttribute('data-s', String(gs));
            t.setAttribute('data-t', String(gt));
            t.textContent = labels.length > 1 ? labels.length + '' : labels[0];
            edgeLayer.appendChild(t);
        });
        root.appendChild(edgeLayer);
        // Internal closed edges per cluster (H26)
        const internal = {};
        EDGES.forEach(e => {
            if (!ids.includes(e.source) || !ids.includes(e.target))
                return;
            const g = byId[e.source].scope + '\u0000' + byId[e.source].type;
            if (g !== byId[e.target].scope + '\u0000' + byId[e.target].type)
                return;
            (internal[g] = internal[g] || []).push(e.label);
        });
        cg.forEach(g => {
            const [scope, type] = g.split('\u0000');
            const gp = cells[g] || { bx: 0, by: 0 };
            const members = cluster[g];
            const w = clusterWidth(g,members.length), h = 34;
            const rect = (document.createElementNS(NS, 'rect') as SVGElement);
            rect.setAttribute('x', String(gp.bx - w / 2));
            rect.setAttribute('y', String(gp.by - h / 2));
            rect.setAttribute('width', String(w));
            rect.setAttribute('height', String(h));
            rect.setAttribute('rx', '8');
            rect.setAttribute('fill', String(KIND_COLORS[type]));
            rect.setAttribute('opacity', '0.6');
            rect.setAttribute('class', 'node');
            rect.setAttribute('role', 'button');
            rect.setAttribute('tabindex', '0');
            rect.setAttribute('aria-label', 'List ' + scope + ' / ' + type + ': ' + members.length + ' records');
            rect.addEventListener('click', () => openClusterList(g));
            rect.addEventListener('keydown', event => {
                if (event.key === 'Enter' || event.key === ' ') { event.preventDefault(); openClusterList(g); }
            });
            root.appendChild(rect);
            const t1 = (document.createElementNS(NS, 'text') as SVGTextElement);
            t1.setAttribute('x', String(gp.bx));
            t1.setAttribute('y', String(gp.by - 1));
            t1.setAttribute('text-anchor', 'middle');
            t1.setAttribute('class', 'nlabel');
            t1.style.fill = token('--color-111');
            t1.textContent = scope + ' / ' + type + ' · ' + members.length;
            root.appendChild(t1);
            const t2 = (document.createElementNS(NS, 'text') as SVGTextElement);
            t2.setAttribute('x', String(gp.bx));
            t2.setAttribute('y', String(gp.by + 13));
            t2.setAttribute('text-anchor', 'middle');
            t2.setAttribute('class', 'elabel');
            t2.style.fill = token('--accent');
            t2.textContent = internal[g] ? ('internal ' + internal[g].length) : '';
            root.appendChild(t2);
        });
        setMeta(cg.length, ids.length + ' nodes', cg.length + ' clusters · ' + ids.length + ' nodes available');
        return;
    }
    // Individual mode / filtered small set: force layout (H21)
    drawEdgesLayer(inPositions, ids, visibleEdges(ids));
    const n = drawNodeLayer(inPositions, ids);
    // H28 + H18: at the readable fit scale some nodes may fall outside the viewport.
    const off = offscreenCount(inPositions, ids);
    const noted = off > 0 ? ('Showing ' + (n - off) + ' of ' + n + ' nodes; ' + off + ' off-screen at readable zoom. Zoom out or pan to reach them.') : '';
    setMeta(n - off, n, noted);
}
export function offscreenCount(layout, ids) {
    const rect = svg.getBoundingClientRect();
    let off = 0;
    ids.forEach(id => {
        const p = layout[id];
        if (!p)
            return;
        const sx = p.x * scale + translate.x;
        const sy = p.y * scale + translate.y;
        if (sx < -20 || sx > rect.width + 20 || sy < -20 || sy > rect.height + 20)
            off++;
    });
    return off;
}
export function drawEdgesLayer(inPositions, ids, drawEdges) {
    const edgeLayer = (document.createElementNS(NS, 'g') as SVGElement);
    const wantLabels = !genealogyMode && ids.length <= EDGE_LABEL_MAX;
    drawEdges.forEach((e, idx) => {
        const a = inPositions[e.source], b = inPositions[e.target];
        if (!a || !b)
            return;
        const line = (document.createElementNS(NS, 'line') as SVGElement);
        const p = dotPath(a.x, a.y, b.x, b.y);
        line.setAttribute('x1', String(p.x1));
        line.setAttribute('y1', String(p.y1));
        line.setAttribute('x2', String(p.x2));
        line.setAttribute('y2', String(p.y2));
        line.setAttribute('stroke', String(edgeColor(e.label)));
        line.setAttribute('stroke-width', '1.2');
        line.setAttribute('marker-end', String('url(#arr-' + e.label.replace(/\W/g, '_') + ')'));
        line.setAttribute('opacity', '0.5');
        line.setAttribute('data-lbl', String(e.label));
        line.setAttribute('data-idx', String(idx));
        edgeLayer.appendChild(line);
        if (wantLabels) {
            const t = (document.createElementNS(NS, 'text') as SVGTextElement);
            t.setAttribute('x', String((p.x1 + p.x2) / 2));
            t.setAttribute('y', String((p.y1 + p.y2) / 2 - 3));
            t.setAttribute('text-anchor', 'middle');
            t.setAttribute('class', 'elabel');
            t.textContent = e.label;
            edgeLayer.appendChild(t);
        }
    });
    root.appendChild(edgeLayer);
}
export function drawNodeLayer(inPositions, ids) {
    const nodeLayer = (document.createElementNS(NS, 'g') as SVGElement);
    let drawn = 0;
    ids.forEach(id => {
        const n = byId[id];
        const p = inPositions[id];
        if (!p)
            return;
        const isSelected = selected === id;
        const isHit = searchHits && searchHits.has(id);
        const g = (document.createElementNS(NS, 'g') as SVGElement);
        g.setAttribute('transform', String('translate(' + p.x + ',' + p.y + ')'));
        g.setAttribute('class', 'node');
        if (isSelected) {
            const ring = (document.createElementNS(NS, 'circle') as SVGElement);
            ring.setAttribute('r', '13');
            ring.setAttribute('class', 'ring');
            ring.setAttribute('fill', 'none');
            ring.setAttribute('stroke', 'var(--hl)');
            ring.setAttribute('stroke-width', '3');
            g.appendChild(ring);
        }
        if (isHit) {
            const hit = (document.createElementNS(NS, 'circle') as SVGElement);
            hit.setAttribute('r', '15');
            hit.setAttribute('fill', 'none');
            hit.setAttribute('stroke', String(token('--color-ffb000')));
            hit.setAttribute('stroke-width', '2');
            g.appendChild(hit);
        }
        const r = lod === 'near' ? 11 : 8;
        const path = (document.createElementNS(NS, 'path') as SVGElement);
        path.setAttribute('d', String(shapeOf(n.type)));
        path.setAttribute('fill', String(isSelected ? token('--hl') : KIND_COLORS[n.type]));
        path.setAttribute('stroke', String(token('--color-222')));
        path.setAttribute('stroke-width', '0.8');
        path.setAttribute('transform', String('scale(' + (r / 8) + ')'));
        g.appendChild(path);
        if (n.type === 'requirement' && n.status === 'complete') {
            const ck = (document.createElementNS(NS, 'text') as SVGTextElement);
            ck.setAttribute('x', String(0));
            ck.setAttribute('y', String(3));
            ck.setAttribute('text-anchor', 'middle');
            ck.style.fontSize = token('--font-9px');
            ck.style.fill = token('--color-fff');
            ck.textContent = '✓';
            g.appendChild(ck);
        }
        if (n.type === 'decision' && n.superseded_by && n.superseded_by.length) {
            const x = (document.createElementNS(NS, 'text') as SVGTextElement);
            x.setAttribute('x', String(0));
            x.setAttribute('y', String(3));
            x.setAttribute('text-anchor', 'middle');
            x.style.fontSize = token('--font-9px');
            x.style.fill = token('--color-fff');
            x.textContent = '×';
            x.setAttribute('transform', 'scale(0.8)');
            g.appendChild(x);
        }
        const label=nodeLabel(id);
        const t=document.createElementNS(NS,'text') as SVGTextElement;
        t.setAttribute('class','nlabel');
        t.setAttribute('text-anchor','middle');
        t.dataset.truncated=String(label.truncated);
        t.dataset.retained=String(label.retained);
        [id,...label.lines].forEach((line,index)=>{
            const span=document.createElementNS(NS,'tspan');
            span.setAttribute('x',index===0?'18':'22');
            if(index===0) span.setAttribute('text-anchor','start');
            span.setAttribute('y',String(index===0?4:26+(index-1)*LINE_HEIGHT));
            span.textContent=line;
            t.appendChild(span);
        });
        g.appendChild(t);
        g.dataset.nodeId = id;
        g.setAttribute('tabindex', '0');
        g.setAttribute('role', 'button');
        g.setAttribute('aria-label', String('Read ' + id + ' · ' + n.title));
        g.addEventListener('click', (e) => { e.stopPropagation(); if (!dragMoved)
            selectNode(id); });
        g.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                selectNode(id);
            }
        });
        nodeLayer.appendChild(g);
        drawn++;
    });
    root.appendChild(nodeLayer);
    return drawn;
}
