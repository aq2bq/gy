(() => {
"use strict";
const D = window.GY_DATA;
const NODES = D.nodes;           // [{id,type,scope,title,status,closed,attrs:{},body,state,created}]
const EDGES = D.edges;           // [{source,target,label,reverse}]
const DEP   = D.dependencies;    // [{requirement,decision,role,superseded_by}]
const EMPTY_BODY = D.bodies ? D.bodies : null;

// draw is defined later; redraw is used by listeners registered before it
function redraw() { draw(); }

const byId = {};
NODES.forEach(n => byId[n.id] = n);

// ---------- helpers ----------
const esc = (s) => String(s).replace(/[&<>"]/g, c => ({'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;'}[c]));
const nowInfo = () => D.generated_at;
const KIND_COLORS = {need:'#7a9e7e',question:'#d9a441',decision:'#4f7fb8',requirement:'#8a6fb2',criterion:'#c96f6f',gate:'#4f9aa8'};
const KIND_SHAPE = {need:'circle',question:'diamond',decision:'rect',requirement:'hexagon',criterion:'triangle',gate:'rect'};

