import { MODE_THRESHOLD, currentFocus, dragMoved, dragStart, dragging, forceLayout, genealogyLayout, genealogyMode, lod, positions, scale, setDragMoved, setDragStart, setDragging, setLastFitKey, setLod, setScale, setTranslate, setViewSource, translate } from '../state';
import { draw, overviewGrid } from './draw';
import { ensureForce } from './layout';
import { lodFromScale, visibleNodes } from './selection';
import { root, svg } from './svg';
export function initViewport() {
    // ---------- pan / zoom ----------
    svg.addEventListener('wheel', (e) => {
        e.preventDefault();
        const factor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
        const rect = svg.getBoundingClientRect();
        const cx = e.clientX - rect.left, cy = e.clientY - rect.top;
        const nx = (cx - translate.x) / scale, ny = (cy - translate.y) / scale;
        setViewSource('manual');
        setScale(scale * (factor));
        setScale(Math.min(4, Math.max(0.1, scale)));
        setTranslate({ ...translate, x: cx - nx * scale });
        setTranslate({ ...translate, y: cy - ny * scale });
        applyTransform();
        updateLod();
        draw();
    }, { passive: false });
    svg.addEventListener('mousedown', (e) => {
        if (e.button !== 0)
            return;
        setDragging(true);
        setDragMoved(false);
        setDragStart({ x: e.clientX, y: e.clientY, tx: translate.x, ty: translate.y });
    });
    window.addEventListener('mousemove', (e) => {
        if (!dragging)
            return;
        if (!dragMoved && Math.hypot(e.clientX - dragStart.x, e.clientY - dragStart.y) < 4)
            return;
        setDragMoved(true);
        setViewSource('manual');
        setTranslate({ ...translate, x: dragStart.tx + e.clientX - dragStart.x });
        setTranslate({ ...translate, y: dragStart.ty + e.clientY - dragStart.y });
        applyTransform();
        updateLod();
        draw();
    });
    window.addEventListener('mouseup', () => { setDragging(false); });
}
export function applyTransform() {
    root.setAttribute('transform', String('translate(' + translate.x + ',' + translate.y + ') scale(' + scale + ')'));
}
export function updateLod() {
    const next = lodFromScale();
    if (next !== lod) {
        setLod(next);
    }
}
export function zoomBy(f) {
    const rect = svg.getBoundingClientRect();
    const cx = rect.width / 2, cy = rect.height / 2;
    const nx = (cx - translate.x) / scale, ny = (cy - translate.y) / scale;
    setViewSource('manual');
    setScale(scale * (f));
    setScale(Math.min(4, Math.max(0.1, scale)));
    setTranslate({ ...translate, x: cx - nx * scale });
    setTranslate({ ...translate, y: cy - ny * scale });
    applyTransform();
    updateLod();
    draw();
}
export function currentLayout() {
    const ids = visibleNodes();
    if (genealogyMode)
        return genealogyLayout;
    if (ids.length <= MODE_THRESHOLD) {
        ensureForce(ids);
        return forceLayout;
    }
    return positions;
}
export function fitTransform() {
    const ids = visibleNodes();
    const over = !genealogyMode && ids.length > MODE_THRESHOLD;
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity;
    if (over) {
        // Fit the cluster rectangles of the overview mode.
        const { cluster, cells } = overviewGrid(ids);
        for (const g of Object.keys(cluster)) {
            const gp = cells[g] || { bx: 0, by: 0 };
            const w = Math.min(280, Math.max(120, 40 + cluster[g].length * 1.4));
            minX = Math.min(minX, gp.bx - w / 2);
            minY = Math.min(minY, gp.by - 20);
            maxX = Math.max(maxX, gp.bx + w / 2);
            maxY = Math.max(maxY, gp.by + 20);
        }
    }
    else {
        const layout = currentLayout();
        const ids2 = ids.filter(id => layout[id]);
        const list = ids2.length ? ids2 : Object.keys(layout);
        list.forEach(id => {
            const p = layout[id];
            if (!p)
                return;
            minX = Math.min(minX, p.x);
            minY = Math.min(minY, p.y);
            maxX = Math.max(maxX, p.x);
            maxY = Math.max(maxY, p.y);
        });
    }
    setViewSource('fit');
    if (minX > maxX)
        return;
    const rect = svg.getBoundingClientRect();
    const spanX = (maxX - minX) + 120;
    const spanY = (maxY - minY) + 120;
    const fit = Math.min(rect.width / Math.max(1, spanX), rect.height / Math.max(1, spanY));
    // Avoid over-enlarging isolated/small neighborhoods; manual zoom still reaches 4.
    setScale(Math.min(over ? 4 : 2, fit));
    // H28: keep node labels readable (font 11px -> at least 11px on screen).
    // If the whole set does not fit at that scale, readability wins and the
    // off-screen count is reported by the banner in draw().
    if (!over)
        setScale(Math.max(scale, 1.0));
    let center = { x: (minX + maxX) / 2, y: (minY + maxY) / 2 };
    const focus = currentFocus();
    // At the readable minimum, keep the destination visible even if its
    // neighborhood spans more than the viewport (notably generation layouts).
    if (!over && fit < 1 && focus.id && ids.includes(focus.id)) {
        center = currentLayout()[focus.id] || center;
    }
    setTranslate({ ...translate, x: rect.width / 2 - center.x * scale });
    setTranslate({ ...translate, y: rect.height / 2 - center.y * scale });
    applyTransform();
    updateLod();
}
export function fitView() { setLastFitKey(''); draw(); }
export function resetView() { fitView(); }
