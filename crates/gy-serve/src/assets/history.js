/* The history page (n-4a08, a region since d-03ca 第 3 段): every write, newest
   first, with the writer and the node it touched. The rows come from the state
   the root read; the writer and the day live in state.history. One row's words
   are GyWrites (n-b963). */
(function () {
  let order = [];
  let seen = null;

  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  /* The rows, grouped by the day they were written. */
  function rows(data, ui) {
    if (!data.rows.length) return `<div class="empty">${ui.t('histEmpty')}</div>`;
    let html = '';
    let head = '';
    for (const entry of data.rows) {
      const today = window.GyWrites.day(entry.at);
      if (today !== head) {
        head = today;
        html += `<div class="hist-day">${head}</div>`;
      }
      html += window.GyWrites.row(entry, data.labels || {}, order);
    }
    if (data.total > data.rows.length) html += `<div class="empty">${ui.t('histOlder')}</div>`;
    return `<div class="hist">${html}</div>`;
  }

  /* The writer chips: "all", then every writer the answer names. data-a stays
     for e2e until 第 4 段 moves it to the act. */
  function chips(state, data, ui) {
    const actor = state.history.actor;
    const all = `<button data-a="" data-act="historyActor" data-arg="" class="${actor === null ? 'on' : ''}">${ui.t('filterActorAll')}</button>`;
    const names = data.actors
      .map(name => `<button data-a="${esc(name)}" data-act="historyActor" data-arg="${esc(name)}" class="${actor === name ? 'on' : ''}">${esc(name)}</button>`)
      .join('');
    return `<div class="chips hist-actors" id="hist-actors">${all}${names}</div>`;
  }

  /* The sequence to ask from: the oldest write on that day, minus one, so the
     day's own writes are kept. No write that late means the whole ledger. Pure,
     so the root's event handler can ask it for a date it was handed (n-ce20). */
  function since(state, value) {
    const data = state.page;
    if (!value || !data) return null;
    const start = new Date(`${value}T00:00:00`).getTime() / 1000;
    const first = data.rows.filter(entry => entry.at >= start).pop();
    return first ? Math.max(0, first.seq - 1) : null;
  }

  /* The region: #main only, from the state the root read. */
  function render(state, el, ui) {
    if (!el) return;
    const data = state.page;
    if (!data) {
      el.innerHTML = '';
      return;
    }
    if (seen !== data) {
      seen = data;
      order = [];
    }
    const scope = state.scope === 'all' ? ui.t('all') : state.scope;
    el.innerHTML =
      `<div class="list"><h1>${ui.t('histTitle')}</h1><div class="sub">${fill(ui.t('histSub'), { n: data.total })} · ${esc(scope)}</div>` +
      `<div class="hist-filter">${chips(state, data, ui)}<label class="hist-date">${ui.t('histDate')}<input type="date" id="hist-date" value=""></label></div>` +
      `${rows(data, ui)}</div>`;
  }

  window.GyHistory = { render, since };
})();