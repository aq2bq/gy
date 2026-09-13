import { renderLint } from './survey/blockers';
import { renderList } from './list';
import { overviewSource, setOverviewSource, listSort, setListSort } from './state';
import { byId } from './data';
import { hideDetail, showDetail } from './detail/index';
import { element } from './dom';
import { gotoTab, renderTypeChips } from './filters';
import { resetView } from './graph/viewport';
import { LocationState, activeTab, currentFocus, filterState, focusHistory, genealogyMode, locationHash, radiusChoice, restoreFocus, searchText, selected, setFilter, setGenealogyMode, setRadiusChoice, setSearchText, setType, typeState } from './state';
// URL state is the shared location contract. Pan and zoom are deliberately absent.
let lastLocationState = '';
export function locationState(value?: LocationState) {
    const fields = ['scopeSel', 'stateSel', 'qstatus', 'criterion'] as const;
    if (value === undefined) {
        return JSON.stringify({ selected, focus: focusHistory, types: typeState,
            filters: filterState,
            overview: overviewSource, listSort, search: searchText, radiusChoice, tab: activeTab, genealogy: genealogyMode });
    }
    restoreFocus(value.focus);
    Object.keys(typeState).forEach(k => { setType(k as keyof typeof typeState, value.types?.[k] !== false); });
    renderTypeChips();
    setListSort(value.listSort);
    setOverviewSource(value.overview);
    renderLint();
    fields.forEach(id => { element(id).value = typeof value.filters?.[id] === 'string' ? value.filters[id] : ''; setFilter(id, element(id).value); });
    element('hopRadius').value = ['', '1', '2', '3', '4', '5'].includes(value.radiusChoice) ? value.radiusChoice : '';
    setRadiusChoice(element('hopRadius').value);
    setSearchText(typeof value.search === 'string' ? value.search : '');
    element('q').value = searchText;
    setGenealogyMode(value.genealogy === true);
    const tabs = [...document.querySelectorAll<HTMLElement>('#tabs button')].map(b => b.dataset.tab);
    gotoTab(tabs.includes(value.tab) ? value.tab : 'overview');
    element('hopFrom').value = currentFocus().id || '';
    hideDetail();
    element('locationStatus').textContent = '';
    if (typeof value.selected === 'string') {
        if (Object.hasOwn(byId, value.selected))
            showDetail(value.selected);
        else
            element('locationStatus').textContent = 'Node not found: ' + value.selected;
    }
}
export function restoreLocation() {
    const state = locationHash();
    locationState(state);
    lastLocationState = locationState();
    resetView();
}
export function saveLocation() {
    const state = locationState();
    if (state === lastLocationState)
        return;
    history.pushState(null, '', locationHash(state));
    lastLocationState = state;
    renderList();
    element('locationStatus').textContent = '';
}
