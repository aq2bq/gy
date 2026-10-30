/* The first screen (d-e239, d-647f): where every open need stands, what it
   waits on, and which root holds the most needs. It draws the frame first and
   the answer when it comes; a refused answer leaves the frame and a reason
   (r-2d02, ac-0dbb). */
(function () {
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));
  const STAGES = ['before', 'build', 'verify', 'close'];
  const DAY = 86400;
  const pad = value => String(value).padStart(2, '0');
  const dateOf = at => {
    const date = new Date(at * 1000);
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
  };
  const md = at => {
    const date = new Date(at * 1000);
    return `${date.getMonth() + 1}/${date.getDate()}`;
  };
  const trunc = (text, n) => (String(text).length > n ? `${String(text).slice(0, n - 1)}…` : String(text));
  const ageCol = days => (days < 3 ? 'var(--decision)' : days < 10 ? 'var(--paper-2)' : days < 21 ? 'var(--need)' : 'var(--stop)');
  const pips = (done, all, max = 6) => (all > max ? `${done}/${all}` : '●'.repeat(done) + '○'.repeat(Math.max(0, all - done)));

  /* The frame's empty hand: the numbers and the board are placeholders until
     the answer comes, so the page never opens blank. */
  const dash = '–';

  function render(state, el, ui) {
    if (!el) return;
    const body = state.overview ? window.GyShape.overview(state.overview) : null;
    const shell = state.shell;
    const ledger = ui.ledger || '';
    const now = body && body.now ? body.now : null;
    const sub = fill(ui.t('overviewSub'), {
      ledger: esc(ledger),
      seq: String((body && body.seq) || (shell && shell.seq) || '—'),
      at: now ? ui.when(now) : '—',
    });
    el.innerHTML = `<div class="sub">${sub}</div><h1 class="hd">${esc(ui.t('overviewTitle'))}</h1>` +
      (body ? stats(body, ui) + board(body, state, ui) + blockers(body, ui) : frame(ui)) +
      status(state, 'overview', !!state.overview, ui);
  }

  function stats(body, ui) {
    const totals = body.totals;
    const stat = (value, label, kind, key) => `<div class="stat ${kind || ''}" data-testid="stat" data-key="${key}"><div class="n" data-testid="statValue">${value}</div><div class="l" data-testid="statLabel">${esc(ui.t(label))}</div></div>`;
    return `<div class="stats" data-testid="stats">` +
      stat(totals.open || body.needs.length, 'statOpen', 'need', 'open') +
      stat(totals.blocked, 'statBlocked', 'stop', 'blocked') +
      stat(totals.questions, 'statQuestions', 'q', 'questions') +
      stat(totals.unmet, 'statUnmet', 'a', 'unmet') +
      stat(totals.done, 'statDone', '', 'done') +
      `<div class="flowchart"><div class="sub">${esc(ui.t('flowSub'))}</div>${flow(body.weeks, ui)}</div></div>`;
  }

  /* Week by week: the needs opened above the line, the needs finished below.
     A bar opens that week on the cross screen. */
  function flow(weeks, ui) {
    if (!weeks.length) return `<div class="dim">${esc(ui.t('noData'))}</div>`;
    const max = Math.max(1, ...weeks.map(week => Math.max(week.opened, week.done)));
    const band = 40;
    const width = Math.max(120, weeks.length * 34 + 10);
    const bars = weeks.map((week, index) => {
      const x = index * 34 + 4;
      const up = (week.opened / max) * band;
      const down = (week.done / max) * band;
      const from = dateOf(week.from);
      const to = week.to ? dateOf(week.to) : dateOf(week.from + 6 * DAY);
      return `<a href="#/cross?from=${from}&to=${to}" data-testid="week"><title>${md(week.from)}: ${week.opened} / ${week.done}</title>` +
        `<rect x="${x}" y="${band - up}" width="26" height="${up}" fill="var(--need)" opacity=".85"/>` +
        `<rect x="${x}" y="${band + 2}" width="26" height="${down}" fill="var(--decision)" opacity=".75"/>` +
        `<text x="${x}" y="${band * 2 + 14}">${md(week.from)}</text></a>`;
    }).join('');
    return `<svg width="${width}" height="${band * 2 + 18}">${bars}</svg>`;
  }

  /* The board: one row per scope, one column per stage. A card that waits wears
     the stop colour and names what it waits on. */
  function board(body, state, ui) {
    const needs = body.needs;
    const scopes = scopeOrder(body);
    const stages = STAGES.map(stage => ({ stage, count: needs.filter(item => item.stage === stage).length }));
    const head = `<div class="ch">${esc(ui.t('scopeCol'))}<small>${esc(ui.t('pulseSub'))}</small></div>` +
      stages.map(item => `<div class="ch">${esc(ui.t(`stage.${item.stage}`))}<span class="cnt">${item.count}</span><small>${esc(ui.t(`stage.${item.stage}Sub`))}</small></div>`).join('');
    let rows = '';
    for (const scope of scopes) {
      const mine = needs.filter(item => item.scope === scope);
      const row = body.scopes.find(item => item.name === scope);
      const done = row ? row.done : 0;
      const blocked = row ? row.blocked : mine.filter(item => item.blocked).length;
      rows += `<div class="rh" style="--h:${ui.hue(scope)}"><div class="name">${esc(scope)}</div>` +
        `<div class="meta">${fill(esc(ui.t('openDoneBlocked')), { o: row ? row.open : mine.length, d: done, b: blocked })}</div>` +
        (row && row.pulse.length ? pulse(row.pulse, scope) : '') +
        `<a class="x" href="#/cross?scope=${encodeURIComponent(scope)}&from=${dateOf((body.now || Date.now() / 1000) - 13 * DAY)}&to=${dateOf(body.now || Date.now() / 1000)}">${esc(ui.t('cross2w'))}</a></div>`;
      for (const stage of STAGES) {
        const cards = mine.filter(item => item.stage === stage).sort(byCard);
        rows += `<div class="cell ${stage === 'before' ? 'wide' : ''}">${cards.length ? cards.map(item => card(item, state, ui)).join('') : `<span class="none">—</span>`}</div>`;
      }
    }
    return `<div class="sec"><h2>${esc(ui.t('boardTitle'))} <span class="sub">${esc(ui.t('boardSub'))}</span></h2><div class="board" data-testid="board">${head}${rows}</div></div>`;
  }

  const byCard = (a, b) => (Number(b.blocked) - Number(a.blocked)) || ((b.last || 0) - (a.last || 0));

  /* The scopes with a need, the busiest first; the overview's own order wins a
     tie so the board is stable. */
  function scopeOrder(body) {
    const counts = {};
    for (const item of body.needs) counts[item.scope] = (counts[item.scope] || 0) + 1;
    return Object.keys(counts).sort((a, b) => counts[b] - counts[a] || a.localeCompare(b));
  }

  /* The writes' pulse: one thin bar a day, square-rooted so a quiet day shows. */
  function pulse(values, scope) {
    const max = Math.max(1, ...values);
    const width = 150;
    const step = width / Math.max(1, values.length);
    const bars = values.map((value, index) => {
      if (!value) return '';
      const height = Math.max(1.5, Math.sqrt(value / max) * 22);
      return `<rect x="${index * step}" y="${22 - height}" width="${Math.max(1, step - 1)}" height="${height}"/>`;
    }).join('');
    return `<svg class="pulse" width="${width}" height="22" style="color:hsl(var(--h) 55% 60%)">${bars}</svg>`;
  }

  function card(item, state, ui) {
    const now = state.overview && state.overview.at ? window.GyShape.at(state.overview.at) : Math.floor(Date.now() / 1000);
    const days = item.last ? Math.max(0, Math.floor((now - item.last) / DAY)) : null;
    const age = days === null ? '' : days === 0 ? ui.t('todayMoved') : fill(ui.t('daysQuiet'), { n: days });
    const questions = item.waiting.filter(one => one.kind === 'Question').length;
    const requirements = item.requirements || {};
    const reqTotal = requirements.filed + requirements.approved + requirements.done;
    const criteria = item.criteria || {};
    const critTotal = criteria.satisfied + criteria.unsatisfied;
    const group = [];
    if (questions) group.push(`${esc(ui.t('shortQuestion'))} <i>${questions}</i>`);
    if (reqTotal) group.push(`${esc(ui.t('shortRequirement'))} <i>${pips(requirements.done, reqTotal)}</i>`);
    if (critTotal) group.push(`${esc(ui.t('shortCriterion'))} <i>${pips(criteria.satisfied, critTotal)}</i>`);
    const waiting = item.waiting.length
      ? `<div class="why">${esc(ui.t('waitingOn'))}: <b>${item.waiting.slice(0, 3).map(one => `${esc(one.title || one.id)}${one.decider ? `（${esc(one.decider)}）` : ''}`).join('、')}</b></div>`
      : '';
    return `<a class="card ${item.blocked ? 'blk' : ''}" data-testid="needCard" data-blocked="${item.blocked ? 'true' : 'false'}" href="#/n/${encodeURIComponent(item.id)}">` +
      `<div class="l1"><span class="id">${esc(item.id)}</span><span class="age" style="color:${days === null ? 'var(--paper-3)' : ageCol(days)}">${esc(age)}</span></div>` +
      `<div class="t">${esc(item.title)}</div>${group.length ? `<div class="g">${group.join('')}</div>` : ''}${waiting}</a>`;
  }

  /* The roots of the jams, the overdue checks, and what nobody holds. */
  function blockers(body, ui) {
    const chips = body.deciders.length
      ? body.deciders.map(item => `<span>${esc(item.name || '?')}<b>${item.count}</b></span>`).join('')
      : `<span>${esc(ui.t('none'))}</span>`;
    const roots = body.roots.slice().sort((a, b) => b.needs.length - a.needs.length);
    const row = item => {
      const head = item.kind === 'Question'
        ? `<span class="who">${esc(fill(ui.t('decides'), { who: item.decider || '?' }))}</span><a href="#/n/${encodeURIComponent(item.id)}">${esc(item.id)} ${esc(trunc(item.title, 40))}</a>`
        : `<a href="#/n/${encodeURIComponent(item.id)}">${esc(item.id)} ${esc(trunc(item.title, 40))}</a>`;
      const needs = item.needs.map(id => `<a href="#/n/${encodeURIComponent(id)}">${esc(id)}</a>`).join('・');
      const via = item.via.length ? ` · ${esc(ui.t('via'))} ${item.via.map(id => `<a href="#/n/${encodeURIComponent(id)}">${esc(id)}</a>`).join(' → ')}` : '';
      return `<div class="chain" data-testid="chain"><div><div class="root">${head}</div><div class="via">${esc(ui.t('stoppedNeeds'))}: ${needs}${via}</div></div>` +
        `<div class="cnt">${item.needs.length}<small>${esc(ui.t('needsUnit'))}</small></div></div>`;
    };
    const mini = (item, right, testid) => `<a class="mini" data-testid="${testid}" href="#/n/${encodeURIComponent(item.id)}"><span class="mono dim">${esc(item.id)}</span><span>${ui.scopeTag(item.scope)}</span>` +
      `<span class="t">${right ? `<span class="mono over">${esc(right)}</span> ` : ''}${esc(item.title)}</span></a>`;
    const overdue = body.overdue.length
      ? body.overdue.map(item => mini(item, `due ${item.due.slice(5)}`, 'overdue')).join('')
      : `<div class="dim">${esc(ui.t('none'))}</div>`;
    const unheld = body.unheld.length
      ? body.unheld.slice(0, 8).map(item => mini(item, null, 'unheld')).join('') +
        (body.unheld.length > 8 ? `<div class="dim" style="font-size:12px;padding-top:4px">${fill(esc(ui.t('moreNodes')), { n: body.unheld.length - 8 })}</div>` : '')
      : `<div class="dim">${esc(ui.t('none'))}</div>`;
    return `<div class="sec blockers"><div><h2>${esc(ui.t('blockersTitle'))} <span class="sub">${esc(ui.t('blockersSub'))}</span></h2>` +
      `<div class="decider">${esc(ui.t('deciderHead'))}: ${chips}</div>` +
      (roots.length ? roots.map(row).join('') : `<div class="dim">${esc(ui.t('noJam'))}</div>`) + `</div>` +
      `<div><h2>${esc(ui.t('overdueTitle'))} <span class="sub">${esc(fill(ui.t('overdueSub'), { n: body.overdue.length }))}</span></h2>${overdue}` +
      `<h2 style="margin-top:22px">${esc(ui.t('unheldTitle'))} <span class="sub">${esc(fill(ui.t('unheldSub'), { n: body.unheld.length }))}</span></h2>${unheld}</div></div>`;
  }

  /* What a screen says while its answer is missing: loading, or why not. */
  function status(state, kind, loaded, ui) {
    if (state.failed[kind]) {
      return `<div class="load fail" data-testid="failed"><b>${esc(ui.t('loadFailed'))}</b> <span>${esc(state.failed[kind])}</span><button data-act="retry">${esc(ui.t('retry'))}</button></div>`;
    }
    if (!loaded) return `<div class="load" data-testid="loading">${esc(ui.t('loading'))}</div>`;
    return '';
  }

  /* The frame before any answer: the same shape, with no claims in it. */
  function frame(ui) {
    const stat = (label, key) => `<div class="stat" data-testid="stat" data-key="${key}"><div class="n" data-testid="statValue">${dash}</div><div class="l" data-testid="statLabel">${esc(ui.t(label))}</div></div>`;
    const head = `<div class="ch">${esc(ui.t('scopeCol'))}</div>` +
      STAGES.map(stage => `<div class="ch">${esc(ui.t(`stage.${stage}`))}<small>${esc(ui.t(`stage.${stage}Sub`))}</small></div>`).join('');
    return `<div class="stats">${stat('statOpen', 'open')}${stat('statBlocked', 'blocked')}${stat('statQuestions', 'questions')}${stat('statUnmet', 'unmet')}${stat('statDone', 'done')}</div>` +
      `<div class="sec"><div class="board" data-testid="board">${head}<div class="rh"><div class="name">…</div></div><div class="cell wide"><span class="none">…</span></div><div class="cell"><span class="none">…</span></div><div class="cell"><span class="none">…</span></div><div class="cell"><span class="none">…</span></div></div></div>`;
  }

  window.GyOverview = { render };
})();
