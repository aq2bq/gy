/* The list page (n-5ca6): one kind's rows from /api/list, with the open/all
   switch that every kind but a decision has. */
(function () {
  const OPEN = {
    Need: ['open'],
    Question: ['open'],
    Criterion: ['unsatisfied'],
    Requirement: ['filed', 'approved'],
  };

  let kind = null;
  let filter = 'open';
  let rows = [];

  const t = key => window.GyShell.t(key);
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  /* Whether a row counts as open: the words the model gave its status. */
  function open(row) {
    return (OPEN[row.kind] || []).includes(row.status);
  }

  function word(row) {
    return row.status ? t(`st.${row.kind}.${row.status}`) : '';
  }

  async function draw(next) {
    kind = next;
    filter = 'open';
    const scope = window.GyShell.scope();
    const only = scope && scope !== 'all' ? `&scope=${encodeURIComponent(scope)}` : '';
    const res = await window.GyShell.read(`/api/list?kind=${encodeURIComponent(kind)}${only}`);
    rows = (await res.json()).rows || [];
    render();
  }

  function render() {
    const switchable = kind !== 'Decision';
    const openCount = rows.filter(open).length;
    const shown = switchable && filter === 'open' ? rows.filter(open) : rows;
    const sorted = shown.slice().sort((a, b) => (a.created < b.created ? 1 : a.created > b.created ? -1 : 0));
    const sub = switchable
      ? fill(t('listSub'), { o: openCount, n: rows.length })
      : `${rows.length}`;
    const chips = switchable
      ? `<div class="chips"><button data-f="open" class="${filter === 'open' ? 'on' : ''}">${t('filterOpen')}</button><button data-f="all" class="${filter === 'all' ? 'on' : ''}">${t('filterAll')}</button></div>`
      : '';
    document.getElementById('main').innerHTML = `<div class="list"><h1>${window.GyShell.plural(kind)}</h1><div class="sub">${sub}</div>${chips}
      <div class="head"><span></span><span>${t('colId')}</span><span>${t('colTitle')}</span><span>${t('scope')}</span><span>${t('status')}</span><span>${t('created')}</span></div>
      <div class="rows">${sorted.length ? sorted.map(row).join('') : `<div class="empty">—</div>`}</div></div>`;
    document.querySelector('.chips')?.addEventListener('click', event => {
      const button = event.target.closest('button');
      if (!button) return;
      filter = button.dataset.f;
      render();
    });
  }

  function row(row_) {
    const label = row_.alias || row_.id;
    return `<a class="row wide" href="#/n/${row_.id}"><span class="dot dot-${row_.kind}"></span><span class="id">${esc(label)}</span><span class="t" title="${esc(row_.title)}">${esc(row_.title)}</span>${window.GyShell.scopeTag(row_.scope)}<span class="st">${esc(word(row_))}</span><span class="sc">${esc(row_.created)}</span></a>`;
  }

  window.GyList = { draw };
})();
