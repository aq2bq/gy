import { EDGES, NODES, byId } from './data';
export interface Focus {
    readonly id: string | null;
    readonly radius: number | null;
}
export interface Point {
    readonly x: number;
    readonly y: number;
}
export type NodeKind = 'need' | 'question' | 'decision' | 'requirement' | 'criterion' | 'gate';
export type Filters = Readonly<Record<'scopeSel' | 'stateSel' | 'qstatus' | 'criterion', string>>;
export interface LocationState {
    overview?: string;
    listSort?: ListSort;
    selected?: string | null;
    focus?: readonly Focus[];
    types?: Readonly<Record<string, boolean>>;
    filters?: Partial<Filters>;
    search?: string;
    radiusChoice?: string;
    tab?: string;
    genealogy?: boolean;
}
// ---------- graph state ----------
export const MODE_THRESHOLD = 60; // above this: overview clusters; at/below: individual force layout
export const HOP_CAP = 5; // adaptive neighborhood ceiling, still clamped to the threshold
export const EDGE_LABEL_MAX = 20; // draw edge labels only when the individual set is this small
export const MAX_LABEL_W = 190; // graph units: label width budget at near zoom
export let searchText = '';
export let searchHits: ReadonlySet<string> | null = null;
// Navigation history is independent of filters, search, and genealogy.
export let focusHistory: readonly Focus[] = [{ id: null, radius: null }];
export function currentFocus() { return focusHistory[focusHistory.length - 1]; }
export let genealogyMode = false;
export let selected: string | null = null;
export let scale = 1;
export let translate: Point = { x: 0, y: 0 };
export let viewSource: 'fit' | 'manual' = 'fit';
export let lod: 'far' | 'mid' | 'near' = 'far';
export let positions: Readonly<Record<string, Point>> = {}; // id -> {x,y}, overview cluster-member grid
export let genealogyLayout: Readonly<Record<string, Point>> = {}; // id -> {x,y}, genealogy (unchanged by H24)
export let forceLayout: Readonly<Record<string, Point>> = {}; // id -> {x,y}, individual mode layout
export let forceKey = ''; // cache key for the current force layout
// adjacency (undirected) built once from the edges the graph actually draws
export const adj = {};
NODES.forEach(n => adj[n.id] = {});
EDGES.forEach(e => {
    (adj[e.source] = adj[e.source] || {})[e.target] = e.label;
    (adj[e.target] = adj[e.target] || {})[e.source] = e.reverse;
});
export const degree = Object.fromEntries(NODES.map(n => [n.id, Object.keys(adj[n.id]).length]));
// ---------- adaptive neighborhood (H27) ----------
export function reachable(start, n) {
    let seen = new Set([start]);
    let frontier = [start];
    for (let i = 0; i < n; i++) {
        const next = [];
        frontier.forEach(id => Object.keys(adj[id] || {}).forEach(t => { if (!seen.has(t)) {
            seen.add(t);
            next.push(t);
        } }));
        frontier = next;
    }
    return seen;
}
// Largest n in [1..HOP_CAP] whose reachable set fits the threshold.
// If even one hop exceeds the threshold, focusedSelection bounds rendering.
export function pickHop(start) {
    for (let n = HOP_CAP; n >= 1; n--) {
        const s = reachable(start, n);
        if (s.size <= MODE_THRESHOLD)
            return { n, size: s.size };
    }
    const s = reachable(start, 1);
    return { n: 1, size: s.size };
}
export let typeState: Readonly<Record<NodeKind, boolean>> = { need: true, question: true, decision: true, requirement: true, criterion: true, gate: true };
export let filterState: Filters = { scopeSel: '', stateSel: '', qstatus: '', criterion: '' };
export let radiusChoice = '';
export let activeTab = 'overview';
export let dragging = false;
export let dragStart: {
    x: number;
    y: number;
    tx: number;
    ty: number;
} | null = null;
export let dragMoved = false;
export let lastFitKey = '';
export let lastViewport: {
    width: number;
    height: number;
} | null = null;
export function enterFocus(id: string, radius: number) {
    if (currentFocus().id !== id || currentFocus().radius !== radius)
        focusHistory = [...focusHistory, { id, radius }];
}
export function truncateFocus(length: number) { focusHistory = focusHistory.slice(0, length); }
export function restoreFocus(value: unknown) {
    const restored: Focus[] = [{ id: null, radius: null }];
    if (Array.isArray(value))
        value.forEach(f => {
            if (f && typeof f.id === 'string' && Object.hasOwn(byId, f.id) && Number.isInteger(f.radius) && f.radius >= 1 && f.radius <= HOP_CAP)
                restored.push({ id: f.id, radius: f.radius });
        });
    focusHistory = restored;
}
export function setType(kind: NodeKind, enabled: boolean) { typeState = { ...typeState, [kind]: enabled }; }
export function setFilter(field: keyof Filters, value: string) { filterState = { ...filterState, [field]: value }; }
export function setSearchText(value: typeof searchText) { searchText = value; }
export function setSearchHits(value: typeof searchHits) { searchHits = value; }
export function setGenealogyMode(value: typeof genealogyMode) { genealogyMode = value; }
export function setSelected(value: typeof selected) { selected = value; }
export function setScale(value: typeof scale) { scale = value; }
export function setTranslate(value: typeof translate) { translate = value; }
export function setViewSource(value: typeof viewSource) { viewSource = value; }
export function setLod(value: typeof lod) { lod = value; }
export function setGenealogyLayout(value: typeof genealogyLayout) { genealogyLayout = value; }
export function setForceLayout(value: typeof forceLayout) { forceLayout = value; }
export function setForceKey(value: typeof forceKey) { forceKey = value; }
export function setRadiusChoice(value: typeof radiusChoice) { radiusChoice = value; }
export function setActiveTab(value: typeof activeTab) { activeTab = value; }
export function setDragging(value: typeof dragging) { dragging = value; }
export function setDragStart(value: typeof dragStart) { dragStart = value; }
export function setDragMoved(value: typeof dragMoved) { dragMoved = value; }
export function setLastFitKey(value: typeof lastFitKey) { lastFitKey = value; }
export function setLastViewport(value: typeof lastViewport) { lastViewport = value; }
export function locationHash(state?: string) {
    const prefix = 'view=';
    if (state !== undefined)
        return '#' + prefix + encodeURIComponent(state);
    try {
        const hash = decodeURIComponent(location.hash.slice(1));
        const value = hash.startsWith(prefix) ? JSON.parse(hash.slice(prefix.length)) : (hash ? { selected: hash } : {});
        return value && typeof value === 'object' ? value : {};
    }
    catch (_) {
        return {};
    }
}
export function focusPlan(id) {
    const radius = Number(radiusChoice);
    return radius ? { n: radius, size: reachable(id, radius).size } : pickHop(id);
}
export function focusLabel(id) {
    if (!Object.hasOwn(byId, id))
        return 'Show neighbors';
    const plan = focusPlan(id);
    return id + ': show ' + plan.n + (plan.n === 1 ? ' hop' : ' hops') + ' · ' + plan.size + (plan.size === 1 ? ' node' : ' nodes');
}
export function setPositions(value: typeof positions) { positions = value; }

export type SortColumn = 'id' | 'type' | 'title' | 'scope' | 'status' | 'created';
export interface ListSort { readonly key: SortColumn; readonly direction: 'asc' | 'desc'; }
export let listSort: ListSort = {key:'id', direction:'asc'};
export function setListSort(value: unknown) {
    const v = value as Partial<ListSort> | null;
    listSort = v && ['id','type','title','scope','status','created'].includes(v.key) && ['asc','desc'].includes(v.direction)
        ? {key:v.key, direction:v.direction} : {key:'id', direction:'asc'};
}
export function selectType(kind: NodeKind | 'all') {
    Object.keys(typeState).forEach(k => setType(k as NodeKind, kind === 'all' || k === kind));
}

export let overviewSource = '';
export function setOverviewSource(value: unknown) { overviewSource = typeof value === 'string' && ['next','lint-error','lint-warn'].includes(value) ? value : ''; }
