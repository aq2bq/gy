/* The three sidebar sets as lists (n-c122): waiting, ready, in progress. The
   rows come from the same /api/now the sidebar counts, so the number in the box
   and the number of rows here are always the same. Only what /api/now carries
   is shown: ID, title, status, and the set's own word. */
(function () {
  const t = key => window.GyShell.t(key);
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
      rows: data => data.ready.map(item => ({
        row: item.row,
        own: fill(t('remaining'), { n: item.targets - item.satisfied, m: item.targets }),
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

  function rowHtml(item) {
    const row = item.row;
    const label = row.alias || row.id;
    return `<a class="row wide" href="#/n/${row.id}"><span class="dot dot-${row.kind}"></span><span class="id">${esc(label)}</span><span class="t" title="${esc(row.title)}">${esc(row.title)}</span><span class="st">${esc(t(`st.${row.kind}.${row.status}`))}</span><span class="own">${esc(item.own)}</span></a>`;
  }

  async function draw(key) {
    const eye = EYES[key];
    if (!eye) return;
    const scope = window.GyShell.scope();
    const only = scope && scope !== 'all' ? `?scope=${encodeURIComponent(scope)}` : '';
    const data = await (await window.GyShell.read(`/api/now${only}`)).json();
    const rows = eye.rows(data);
    const own = eye.own ? t(eye.own) : '';
    document.getElementById('main').innerHTML =
      `<div class="list eye-list"><h1>${t(eye.head)}</h1><div class="sub">${rows.length}</div>` +
      `<div class="head"><span></span><span>${t('colId')}</span><span>${t('colTitle')}</span><span>${t('status')}</span><span>${esc(own)}</span></div>` +
      `<div class="rows">${rows.length ? rows.map(rowHtml).join('') : `<div class="empty">—</div>`}</div></div>`;
  }

  window.GyEye = { draw };
})();