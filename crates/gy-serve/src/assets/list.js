/* The list page (n-5ca6, a region since d-03ca 第 3 段): one kind's rows from
   the state the root read, with the open/all switch whose choice lives in
   state.list.filter. It draws inside #main and reads only what it is handed. */
(function () {
  const OPEN = {
    Need: ['open'],
    Question: ['open'],
    Criterion: ['unsatisfied'],
    Requirement: ['filed', 'approved'],
  };

  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  /* Whether a row counts as open: the words the model gave its status. */
  function open(row) {
    return (OPEN[row.kind] || []).includes(row.status);
  }

  function rowHtml(row, ui) {
    const label = row.alias || row.id;
    const word = row.status ? ui.t(`st.${row.kind}.${row.status}`) : '';
    const mark = row.unwaited ? ` <i class="unwaited">${esc(ui.t('unwaited'))} · ${esc(window.GyTime.age(row.created, ui.t))}</i>` : '';
    return `<a class="row wide" href="#/n/${row.id}"><span class="dot dot-${row.kind}"></span><span class="id">${esc(label)}</span><span class="t" title="${esc(row.title)}">${esc(row.title)}${mark}</span>${ui.scopeTag(row.scope)}<span class="st">${esc(word)}</span><span class="sc">${esc(window.GyTime.date(row.created))}</span></a>`;
  }

  /* The chips name the act events.js reads and the act it carries (第 4 段 took
     the old mark away). */
  function chips(filter, ui) {
    const chip = (value, key) =>
      `<button data-act="listFilter" data-arg="${value}" class="${filter === value ? 'on' : ''}">${ui.t(key)}</button>`;
    return `<div class="chips">${chip('open', 'filterOpen')}${chip('all', 'filterAll')}</div>`;
  }

  /* The region: #main only, from the state the root read. */
  function render(state, el, ui) {
    if (!el) return;
    const kind = state.route.arg;
    const rows = state.page ? state.page.rows || [] : [];
    const filter = state.list.filter;
    const switchable = kind !== 'Decision';
    const openCount = rows.filter(open).length;
    const shown = switchable && filter === 'open' ? rows.filter(open) : rows;
    const sorted = shown.slice().sort((a, b) => (a.created < b.created ? 1 : a.created > b.created ? -1 : 0));
    const sub = switchable ? fill(ui.t('listSub'), { o: openCount, n: rows.length }) : `${rows.length}`;
    el.innerHTML = `<div class="list"><h1>${ui.plural(kind)}</h1><div class="sub">${sub}</div>${switchable ? chips(filter, ui) : ''}
      <div class="head"><span></span><span>${ui.t('colId')}</span><span>${ui.t('colTitle')}</span><span>${ui.t('scope')}</span><span>${ui.t('status')}</span><span>${ui.t('created')}</span></div>
      <div class="rows">${sorted.length ? sorted.map(row => rowHtml(row, ui)).join('') : `<div class="empty">—</div>`}</div></div>`;
  }

  window.GyList = { render };
})();