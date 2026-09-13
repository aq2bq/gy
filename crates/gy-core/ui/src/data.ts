export interface LedgerNode {
    id: string;
    type: string;
    scope: string;
    title: string;
    attrs: Record<string, any>;
    [field: string]: any;
}
export interface LedgerEdge {
    source: string;
    target: string;
    label: string;
    reverse: string;
}
export interface Payload {
    nodes: LedgerNode[];
    edges: LedgerEdge[];
    states: Record<string, number>;
    [field: string]: any;
}
// The generated template is the only writer of this existing payload boundary.
export const D = (window as unknown as {
    readonly GY_DATA: Payload;
}).GY_DATA;
export const NODES = D.nodes; // [{id,type,scope,title,status,closed,attrs:{},body,state,created}]
export const EDGES = D.edges; // [{source,target,label,reverse}]
export const DEP = D.dependencies; // [{requirement,decision,role,superseded_by}]
export const byId: Record<string, LedgerNode> = {};
NODES.forEach(n => byId[n.id] = n);
// ---------- helpers ----------
export const states = D.states || {};
export const lintArr = D.lint || [];
