/* One write, drawn the same way on the history page and in the rail (n-b963).
   The words and the language come from the caller (n-88b2); the rows and their
   order come from /api/history, the aliases and titles from /api/labels. */
(function () {
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

  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));

  /* The write's verb, from what the writer asked for (the why). */
  function verb(entry, t) {
    for (const [prefix, key] of VERBS) {
      if (entry.why.startsWith(prefix)) return t(`why.${key}`);
    }
    if (entry.why.startsWith('link')) {
      return `${t('why.link')} · ${entry.why.replace(/^link \S+ /, '')}`;
    }
    return entry.why;
  }

  /* Who did what to which node, in this language. */
  function what(entry, labels, ui, lang) {
    const who = esc(entry.actor);
    const doing = esc(verb(entry, ui.t));
    if (!entry.node) return `${who} ${doing}`;
    const known = labels[entry.node];
    const node = `<span class="nd-name">${esc((known && known.alias) || entry.node)}</span>`;
    const title = known ? ` <span class="tt">${esc(known.title)}</span>` : '';
    return lang === 'ja' ? `${who} が ${node} ${doing}${title}` : `${who} ${doing} ${node}${title}`;
  }

  /* The actor colour: the four colours in the order the writers appear. The
     caller keeps the order, so one page's colours stay put across draws. */
  function whoCls(name, order) {
    let index = order.indexOf(name);
    if (index < 0) {
      order.push(name);
      index = order.length - 1;
    }
    return `a-${Math.min(3, index)}`;
  }

  const when = (at, lang) =>
    new Date(at * 1000).toLocaleTimeString(lang === 'ja' ? 'ja-JP' : 'en-GB', {
      hour: '2-digit', minute: '2-digit',
    });

  const day = at => {
    const date = new Date(at * 1000);
    const two = value => String(value).padStart(2, '0');
    return `${date.getFullYear()}/${two(date.getMonth() + 1)}/${two(date.getDate())}`;
  };

  /* One row: a link to the node when the write has one, so the whole row is a
     hit. The wrapper carries the list item's role and the link keeps its own,
     so the screen reader is told both (n-bc89); a row with no node is plain. */
  function row(entry, labels, order, ui, lang) {
    const body =
      `<span class="when">${when(entry.at, lang)} · ${entry.seq}</span>` +
      `<span class="who ${whoCls(entry.actor, order)}">${esc(entry.actor)}</span>` +
      `<span class="nd"><div class="what">${what(entry, labels, ui, lang)}</div><div class="src" title="${esc(entry.source)}">${esc(entry.source)}</div><div class="why">${esc(entry.why)}</div></span>`;
    return entry.node
      ? `<div role="listitem"><a class="hi" href="#/n/${entry.node}">${body}</a></div>`
      : `<div role="listitem"><div class="hi">${body}</div></div>`;
  }

  window.GyWrites = { row, when, day };
})();