import { esc } from './components';
import { selectNode } from './detail/index';
import { closeClusterPanel, panel } from './graph/clusters';
import { clusterSort, setClusterSort } from './state';
// Current cluster-member list; N-22 will add the filtered list view here.
export function renderClusterRows(rows, type, scope, g) {
    const sorter = clusterSort;
    const list = [...rows].sort((a, b) => {
        if (sorter === 'deg')
            return b.deg - a.deg || (a.id < b.id ? -1 : 1);
        if (sorter === 'id')
            return (a.id < b.id ? -1 : 1);
        return (a.title < b.title ? -1 : a.title > b.title ? 1 : 0) || (a.id < b.id ? -1 : 1);
    });
    panel.innerHTML =
        '<button class="close" data-close="1">&times;</button>' +
            '<h3>' + esc(scope) + ' / ' + esc(type) + ' — ' + list.length + ' nodes</h3>' +
            '<div class="sortrow">' +
            '<button data-sort="deg"' + (sorter === 'deg' ? ' style="border-color:var(--accent)"' : '') + '>by degree</button>' +
            '<button data-sort="title"' + (sorter === 'title' ? ' style="border-color:var(--accent)"' : '') + '>by title</button>' +
            '<button data-sort="id"' + (sorter === 'id' ? ' style="border-color:var(--accent)"' : '') + '>by id</button>' +
            '</div>' +
            '<ul>' + list.map(r => '<li data-go="' + esc(r.id) + '"><span class="id">' + esc(r.id) + '</span>' +
            (r.alive ? '<span class="badge">current</span>' : '') +
            '<span>' + esc(r.title.length > 46 ? r.title.slice(0, 45) + '…' : r.title) + '</span>' +
            '<span class="deg">' + r.deg + '</span></li>').join('') + '</ul>';
    panel.querySelectorAll<HTMLButtonElement>('button[data-sort]').forEach(b => b.addEventListener('click', () => {
        setClusterSort(b.dataset.sort);
        renderClusterRows(rows, type, scope, g);
    }));
    panel.querySelectorAll('li').forEach(li => li.addEventListener('click', () => {
        closeClusterPanel();
        selectNode(li.dataset.go);
    }));
    panel.querySelector('[data-close]').addEventListener('click', closeClusterPanel);
}
