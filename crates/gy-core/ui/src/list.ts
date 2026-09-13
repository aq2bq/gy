import { esc, redraw } from './components';
import { LedgerNode } from './data';
import { selectNode } from './detail/index';
import { element } from './dom';
import { filteredNodes } from './filters/query';
import { locationState } from './location';
import { listSort, locationHash, selected, setListSort, SortColumn } from './state';

const columns: [SortColumn, string][] = [['id','ID'],['type','Type'],['title','Title'],['scope','Scope'],['status','State'],['created','Created']];
const compare = new Intl.Collator('en', {numeric:true, sensitivity:'base'});
let renderedKey = '';
let linkedState = '';
function value(node: LedgerNode, key: SortColumn): string {
    if (key === 'status') {
        if (node.type === 'requirement') return node.state || node.status || '';
        if (node.type === 'question') return node.status === 'closed' ? 'Closed' : 'Open';
        if (node.type === 'criterion') return node.attrs.satisfied ? 'Satisfied' : 'Not satisfied';
        return node.status || '';
    }
    return String(node[key] ?? '');
}
export function initList() {
    element('recordList').querySelector('thead tr').innerHTML = columns.map(([key,label]) =>
        '<th scope="col" data-column="'+key+'"><button data-sort="'+key+'" aria-label="Sort by '+label+'">'+label+'</button></th>').join('');
    element('recordList').addEventListener('click', event => {
        const target = event.target as Element;
        const sort = target.closest<HTMLElement>('[data-sort]');
        if (sort) {
            const key = sort.dataset.sort as SortColumn;
            setListSort({key, direction:listSort.key === key && listSort.direction === 'asc' ? 'desc' : 'asc'});
            redraw();
            return;
        }
        const link = target.closest<HTMLAnchorElement>('a[data-record]');
        if (link && event.button === 0 && !event.ctrlKey && !event.metaKey && !event.shiftKey && !event.altKey) {
            event.preventDefault();
            selectNode(link.dataset.record);
        }
    });
}
export function renderList() {
    const nodes = filteredNodes();
    element('listCount').textContent = nodes.length + (nodes.length === 1 ? ' result' : ' results');
    const key = JSON.stringify([nodes.map(n=>n.id), listSort]);
    if (key !== renderedKey) {
        const rows = [...nodes].sort((a,b) => {
            const order = compare.compare(value(a,listSort.key), value(b,listSort.key));
            return order * (listSort.direction === 'asc' ? 1 : -1) || compare.compare(a.id,b.id);
        });
        element('recordList').querySelector('tbody').innerHTML = rows.map(node => '<tr data-record-id="'+esc(node.id)+'">'+columns.map(([column]) =>
            '<td data-column="'+column+'">'+(column === 'id' ? '<a data-record="'+esc(node.id)+'" href="#'+esc(encodeURIComponent(node.id))+'">'+esc(node.id)+'</a>' : esc(value(node,column)))+'</td>').join('')+'</tr>').join('');
        element('listEmpty').hidden = nodes.length !== 0;
        renderedKey = key;
        linkedState = '';
    }
    element('recordList').querySelectorAll<HTMLElement>('th[data-column]').forEach(th => {
        const direction = th.dataset.column === listSort.key ? (listSort.direction === 'asc' ? 'ascending' : 'descending') : 'none';
        th.setAttribute('aria-sort', direction);
    });
    const snapshot = locationState();
    if (snapshot !== linkedState) {
        const view = JSON.parse(snapshot);
        element('recordList').querySelectorAll<HTMLAnchorElement>('a[data-record]').forEach(link => {
            link.href = locationHash(JSON.stringify({...view,selected:link.dataset.record}));
            link.closest('tr').classList.toggle('selected', link.dataset.record === selected);
            if (link.dataset.record === selected) link.setAttribute('aria-current','true');
            else link.removeAttribute('aria-current');
        });
        linkedState = snapshot;
    }
}
export function revealRow(id: string) {
    const row = [...element('recordList').querySelectorAll<HTMLAnchorElement>('a[data-record]')].find(link=>link.dataset.record === id);
    if (row) {
        row.focus({preventScroll:true});
        row.scrollIntoView({block:'nearest',inline:'nearest'});
    }
}
