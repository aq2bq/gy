/* The history page (n-4a08): every write, newest first, with the writer and
   the node it touched. The rows and their order come from /api/history; the
   aliases and titles from /api/labels. */
(function () {
  const LIMIT = 200;
  const VERBS = [
    ['need add', 'needadd'],
    ['need close', 'needclose'],
    ['question add', 'qadd'],
    ['question close', 'qclose'],
    ['criterion satisfy', 'acsat'],
    ['criterion add', 'acadd'],
    ['decide', 'decide'],
    ['edit', 'edit'],
    ['req ', 'req'],
  ];

  let data = null;
  let labels = {};
  let actor = null;
  let since = null;
  let order = [];

  const t = key => window.GyShell.t(key);
  const lang = () => window.GyShell.lang();
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  /* The scope in view, as its name or "all scopes". */
  function scopeName() {
    return window.GyShell.scope() === 'all' ? t('all') : window.GyShell.scope();
  }

  /* The write's verb, from what the writer asked for (the why). */
  function verb(entry) {
    for (const [prefix, key] of VERBS) {
      if (entry.why.startsWith(prefix)) return t(`why.${key}`);
    }
    if (entry.why.startsWith('link')) {
      return `${t('why.link')} · ${entry.why.replace(/^link \S+ /, '')}`;
    }
    return entry.why;
  }

  const labelOf = id => (labels[id] && labels[id].alias) || id;

  /* Who did what to which node, in this language. */
  function what(entry) {
    const who = esc(entry.actor);
    const doing = esc(verb(entry));
    if (!entry.node) return `${who} ${doing}`;
    const node = `<a href="#/n/${entry.node}">${esc(labelOf(entry.node))}</a>`;
    const title = labels[entry.node] ? ` <span class="tt">${esc(labels[entry.node].title)}</span>` : '';
    return lang() === 'ja' ? `${who} が ${node} ${doing}${title}` : `${who} ${doing} ${node}${title}`;
  }

  /* The actor colour: the four colours in the order the writers appear. */
  function whoCls(name) {
    let index = order.indexOf(name);
    if (index < 0) {
      order.push(name);
      index = order.length - 1;
    }
    return `a-${Math.min(3, index)}`;
  }

  const when = at =>
    new Date(at * 1000).toLocaleTimeString(lang() === 'ja' ? 'ja-JP' : 'en-GB', {
      hour: '2-digit', minute: '2-digit',
    });

  const day = at => {
    const date = new Date(at * 1000);
    const two = value => String(value).padStart(2, '0');
    return `${date.getFullYear()}/${two(date.getMonth() + 1)}/${two(date.getDate())}`;
  };

  function row(entry) {
    return `<div class="hi"><span class="when">${when(entry.at)} · ${entry.seq}</span><span class="who ${whoCls(entry.actor)}">${esc(entry.actor)}</span><span class="nd"><div class="what">${what(entry)}</div><div class="src" title="${esc(entry.source)}">${esc(entry.source)}</div><div class="why">${esc(entry.why)}</div></span></div>`;
  }

  /* The rows, grouped by the day they were written. */
  function rows() {
    if (!data.rows.length) return `<div class="empty">${t('emptyWrites')}</div>`;
    let html = '';
    let head = '';
    for (const entry of data.rows) {
      const today = day(entry.at);
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