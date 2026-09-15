/* The now page (n-b709): the heading, the three sections, and the pulse. It
   reads /api/now and draws into main#main; the words come from GyShell. */
(function () {
  const SECTIONS = [
    ['in_progress', 'inprogress', 'Need', 'emptyNeeds'],
    ['open_questions', 'openq', 'Question', 'emptyQ'],
    ['unmet', 'unmet', 'Criterion', 'emptyAC'],
  ];
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
  let actors = [];

  const t = key => window.GyShell.t(key);
  const lang = () => window.GyShell.lang();
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  /* The scope in view, as its name or "all scopes". */
  function scopeName() {
    return data.scope || t('all');
  }

  /* The label a row shows: the first alias, else the id. */
  function label(row) {
    return row.alias || row.id;
  }

  /* The state word of a row, in this language. */
  function status(row) {
    return t(`st.${row.kind}.${row.status}`);
  }

  function rowHtml(row) {
    return `<a class="row" href="#/n/${row.id}"><span class="dot dot-${row.kind}"></span><span class="id">${esc(label(row))}</span><span class="t" title="${esc(row.title)}">${esc(row.title)}</span><span class="st">${esc(status(row))}</span></a>`;
  }

  /* The first four waits become cards; a requirement shows its reference
     where a question shows its options. */
  function card(item) {
    const row = item.row;
    const who = item.waiting === 'question' ? `${esc(label(row))} · ${t('decider')} ${esc(item.decider)}` : esc(label(row));
    const body = item.waiting === 'question'
      ? `<ol class="opts">${item.options.map(option => `<li>${esc(option)}</li>`).join('')}</ol>`
      : `<div class="opts">${esc(item.reference || '')}</div>`;
    return `<a class="wait" href="#/n/${row.id}"><div class="who">${who}</div><div class="t">${esc(row.title)}</div>${body}</a>`;
  }

  function hero() {
    const waiting = data.waiting;
    const eyebrow = `<div class="eyebrow">${t('waiting')}</div>`;
    if (!waiting.length) {
      const count = data.in_progress.length;
      const paragraph = count
        ? fill(t('nothingP'), { s: scopeName(), n: count })
        : fill(t('nothingPEmpty'), { s: scopeName() });
      return `<div class="hero">${eyebrow}<h1>${t('nothing')}</h1><p>${paragraph}</p></div>`;
    }
    const cards = waiting.slice(0, 4).map(card).join('');
    const rest = waiting.slice(4).map(item => rowHtml(item.row)).join('');
    return `<div class="hero">${eyebrow}<h1>${fill(t('waitingH'), { n: waiting.length })}</h1></div>` +
      cards + (rest ? `<div class="rows" style="margin-top:14px">${rest}</div>` : '');
  }

  function sections() {
    return SECTIONS.map(([field, title, kind, empty]) => {
      const rows = data[field];
      const body = rows.length
        ? rows.slice(0, 8).map(rowHtml).join('')
        : `<div class="empty">${t(empty)}</div>`;
      return `<section class="sec"><h2>${t(title)}<span class="cnt">${rows.length}</span><a class="more" href="#/list/${kind}">${t('seeall')}</a></h2><div class="rows">${body}</div></section>`;
    }).join('');
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

  /* Who did what to which node, in this language. */
  function what(entry) {
    const actor = esc(entry.actor);
    const doing = esc(verb(entry));
    const node = entry.node ? `<a href="#/n/${entry.node}">${esc(entry.node)}</a>` : '';
    if (!node) return `${actor} ${doing}`;
    return lang() === 'ja'
      ? `${actor} が ${node} ${doing}`
      : `${actor} ${doing} ${node}`;
  }

  function line(entry) {
    return `<div class="when">${when(entry.at)} · seq ${entry.seq} · ${esc(entry.actor)}</div><div class="what">${what(entry)}</div><div class="src" title="${esc(entry.source)}">${esc(entry.source)}</div>`;
  }

  /* The dot colour: the four actor colours in the order the writers appear. */
  function actorCls(actor) {
    let index = actors.indexOf(actor);
    if (index < 0) {
      actors.push(actor);
      index = actors.length - 1;
    }
    return `a-${Math.min(3, index)}`;
  }

  function when(at) {
    return new Date(at * 1000).toLocaleTimeString(lang() === 'ja' ? 'ja-JP' : 'en-GB', {
      hour: '2-digit', minute: '2-digit',
    });
  }

  function pulse() {
    const rows = data.recent.map((entry, index) => {
      const cls = `pi ${actorCls(entry.actor)}${index === 0 ? ' new' : ''}`;
      return `<div class="${cls}">${line(entry)}</div>`;
    }).join('');
    const upTo = data.recent.length ? when(data.recent[0].at) : '';
    const sub = fill(t('pulseSub'), { n: data.recent.length, t: upTo });
    return `<aside class="pulse"><h2>${t('pulse')}</h2><div class="sub">${sub}</div><div class="pl">${rows}</div></aside>`;
  }

  async function draw() {
    const scope = window.GyShell.scope();
    const only = scope && scope !== 'all' ? `?scope=${encodeURIComponent(scope)}` : '';
    const res = await fetch(`/api/now${only}`);
    data = await res.json();
    actors = [];
    document.getElementById('main').innerHTML =
      `<div class="home"><div>${hero()}${sections()}</div>${pulse()}</div>`;
  }

  window.GyNow = { draw };
})();
