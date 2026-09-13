// CSS is loaded before this script. Cache palette reads outside render loops.
const values = getComputedStyle(document.documentElement);
export function token(name: string): string { return values.getPropertyValue(name).trim(); }
export const KIND_COLORS = { need: token('--need'), question: token('--question'), decision: token('--decision'), requirement: token('--requirement'), criterion: token('--criterion'), gate: token('--gate') };
export const KIND_SHAPE = { need: 'circle', question: 'diamond', decision: 'rect', requirement: 'hexagon', criterion: 'triangle', gate: 'rect' };
