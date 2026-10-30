/* The three sidebar sets as lists (n-c122, a region since d-03ca 第 3 段): the
   rows come from the state the root read, the same /api/now the sidebar counts,
   so the number in the box and the rows here are always the same. It draws
   inside #main; only what /api/now carries is shown. */
(function () {
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));
  const tail = text => (text ? String(text).split('/').filter(Boolean).pop() || '' : '');

  /* Each set: the heading word, the own column's heading word, and the rows.
     `wait` has two kinds, so a question gives its decider and a requirement
     gives nothing. `next` and `resume` keep the order /api/now sent. */
  const EYES = {
    wait: {
      head: 'eyeWait',
      own: 'decider',
      rows: data => data.waiting.map(item => ({
        row: item.row,
        own: item.waiting === 'question' ? item.decider : '',
      })),
    },
    next: {
      head: 'readyHead',
      own: '',
      rows: (data, ui) => data.ready.map(item => ({
        row: item.row,
        own: fill(ui.t('remaining'), { n: item.targets - item.satisfied, m: item.targets }),
      })),
    },
    resume: {
      head: 'resumeHead',
      own: 'reqRef',
      rows: data => data.resume.in_progress.map(item => ({
        row: { id: item.id, alias: null, kind: 'Requirement', title: item.title, status: item.state },
        own: tail(item.reference),
      })),
    },
  };

  function rowHtml(item, ui) {
    const row = item.row;
    const label = row.alias || row.id;
    return `<a class="row wide" href="#/n/${row.id}"><span class="dot dot-${row.kind}"></span><span class="id">${esc(label)}</span><span class="t" title="${esc(row.title)}">${esc(row.title)}</span><span class="st">${esc(ui.t(`st.${row.kind}.${row.status}`))}</span><span class="own">${esc(item.own)}</span></a>`;
  }

  /* The region: #main only, from the state the root read. */
  function render(state, el, ui) {
    if (!el) return;
    const eye = EYES[state.route.arg];
    const data = state.now;
    if (!eye || !data) {
      el.innerHTML = '';
      return;
    }
    const rows = eye.rows(data, ui);
    const own = eye.own ? ui.t(eye.own) : '';
    el.innerHTML =
      `<div class="list eye-list"><h1>${ui.t(eye.head)}</h1><div class="sub">${rows.length}</div>` +
      `<div class="head"><span></span><span>${ui.t('colId')}</span><span>${ui.t('colTitle')}</span><span>${ui.t('status')}</span><span>${esc(own)}</span></div>` +
      `<div class="rows">${rows.length ? rows.map(item => rowHtml(item, ui)).join('') : `<div class="empty">—</div>`}</div></div>`;
  }

  window.GyEye = { render };
})();