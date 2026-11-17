/* The history page (n-4a08, a region since d-03ca 第 3 段): every write, newest
   first, with the writer and the node it touched. The rows come from the state
   the root read; the writer and the day live in state.history. One row's words
   are GyWrites (n-b963). */
(function () {
  let order = [];
  let seen = null;
  /* How many writers stay out of the fold. */
  const SHOWN = 3;

  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  /* The rows, grouped by the day they were written. */
  function rows(state, data, ui) {
    if (!data.rows.length) return `<div class="empty">${ui.t('histEmpty')}</div>`;
    let html = '';
    let head = '';
    for (const entry of data.rows) {
      const today = window.GyTime.date(entry.at);
      if (today !== head) {
        head = today;
        html += `<div class="hist-day" data-testid="hist-day">${head}</div>`;
      }
      html += window.GyWrites.row(entry, data.labels || {}, order, ui, state.lang);
    }
    if (data.total > data.rows.length) html += `<div class="empty">${ui.t('histOlder')}</div>`;
    return `<div class="hist" role="list">${html}</div>`;
  }

  /* The writer chips: "all", then the writers the answer names; the act events.js
     reads is the only marker (第 4 段 took the old mark away). */
  function chips(state, data, ui) {
    const actor = state.history.actor;
    const chip = name => `<button data-act="historyActor" data-arg="${esc(name)}" class="${actor === name ? 'on' : ''}">${esc(name)}</button>`;
    const all = `<button data-act="historyActor" data-arg="" class="${actor === null ? 'on' : ''}">${ui.t('filterActorAll')}</button>`;
    /* One row (n-fd56): the chosen writer first, then the answer's order up to
       SHOWN; the rest fold under a native disclosure, which a choice closes
       again because the region is redrawn. */
    const chosen = data.actors.filter(name => name === actor);
    const others = data.actors.filter(name => name !== actor);
    const shown = chosen.concat(others.slice(0, SHOWN - chosen.length));
    const rest = others.slice(SHOWN - chosen.length);
    const more = rest.length
      ? `<details class="hist-more" data-testid="histMore"><summary>${esc(fill(ui.t('histMoreActors'), { n: rest.length }))}</summary>` +
        `<div class="rest">${rest.map(chip).join('')}</div></details>`
      : '';
    return `<div class="chips hist-actors" id="hist-actors">${all}${shown.map(chip).join('')}${more}</div>`;
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
      `${rows(state, data, ui)}</div>`;
  }

  window.GyHistory = { render, since };
})();