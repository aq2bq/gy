import { renderLint } from '../survey/blockers';
import { setOverviewSource } from '../state';
import { redraw } from '../components';
import { element } from '../dom';
import { renderTypeChips } from '../filters';
import { revealRow } from '../list';
import { NodeKind, selectType, setFilter } from '../state';

export function openClusterList(group: string) {
    setOverviewSource('');
    renderLint();
    const [scope, type] = group.split('\u0000');
    selectType(type as NodeKind);
    setFilter('scopeSel', scope);
    element('scopeSel').value = scope;
    renderTypeChips();
    redraw();
    element('listPane').scrollTop = 0;
}

export function revealOmitted(members: string[]) {
    if (members.length) revealRow(members[0]);
}
