/* The history page (n-4a08): every write, newest first, with the writer and
   the node it touched. The rows and their order come from /api/history; the
   aliases and titles from /api/labels. One row's words are GyWrites (n-b963). */
(function () {
  const LIMIT = 200;

  let data = null;
  let labels = {};
  let actor = null;
  let since = null;
  let order = [];

  const t = key => window.GyShell.t(key);
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  /* The scope in view, as its name or "all scopes". */
  function scopeName() {
    return window.GyShell.scope() === 'all' ? t('all') : window.GyShell.scope();
  }

  const row = entry => window.GyWrites.row(entry, labels, order);

  /* The rows, grouped by the day they were written. */
  function rows() {
    if (!data.rows.length) return `<div class="empty">${t('histEmpty')}</div>`;
    let html = '';
    let head = '';
    for (const entry of data.rows) {
      const today = window.GyWrites.day(entry.at);
      if (today !== head) {
        head = today;
        html += `<div class="hist-day">${head}</div>`;
      }
      html += row(entry);
    }
    if (data.total > data.rows.length) html += `<div class="empty">${t('histOlder')}</div>`;
    return `<div class="hist">${html}</div>`;
  }

  function chips() {
    const all = `<button data-a="" class="${actor === null ? 'on' : ''}">${t('filterActorAll')}</button>`;
    return `<div class="chips hist-actors" id="hist-actors">${all}${data.actors
      .map(name => `<button data-a="${esc(name)}" class="${actor === name ? 'on' : ''}">${esc(name)}</button>`)
      .join('')}</div>`;
  }

  function filterBar() {
    return `<div class="hist-filter">${chips()}<label class="hist-date">${t('histDate')}<input type="date" id="hist-date" value=""></label></div>`;
  }

  function render() {
    document.getElementById('main').innerHTML =
      `<div class="list"><h1>${t('histTitle')}</h1><div class="sub">${fill(t('histSub'), { n: data.total })} · ${esc(scopeName())}</div>${filterBar()}${rows()}</div>`;
    const input = document.getElementById('hist-date');
    input.addEventListener('change', () => {
      since = input.value ? dayStart(input.value) : null;
      draw();
    });
    document.getElementById('hist-actors').addEventListener('click', event => {
      const button = event.target.closest('button[data-a]');
      if (!button) return;
      actor = button.dataset.a || null;
      draw();
    });
  }

  /* The sequence to ask from: the oldest write on that day, minus one, so the
     day's own writes are kept. No write that late means the whole ledger. */
  function dayStart(value) {
    const start = new Date(`${value}T00:00:00`).getTime() / 1000;
    const first = data.rows.filter(entry => entry.at >= start).pop();
    return first ? Math.max(0, first.seq - 1) : null;
  }

  async function fetchLabels(ids) {
    const unique = [...new Set(ids.filter(Boolean))];
    for (let index = 0; index < unique.length; index += LIMIT) {
      const chunk = unique.slice(index, index + LIMIT);
      const res = await window.GyShell.read(`/api/labels?ids=${chunk.join(',')}`);
      Object.assign(labels, (await res.json()).labels);
    }
  }

  async function draw() {
    const scope = window.GyShell.scope();
    const parts = [];
    if (scope && scope !== 'all') parts.push(`scope=${encodeURIComponent(scope)}`);
    if (actor) parts.push(`actor=${encodeURIComponent(actor)}`);
    if (since !== null) parts.push(`since=${since}`);
    const res = await window.GyShell.read(`/api/history${parts.length ? `?${parts.join('&')}` : ''}`);
    data = await res.json();
    order = [];
    await fetchLabels(data.rows.map(entry => entry.node));
    render();
  }

  window.GyHistory = { draw };
})();