/* The now page (d-3e8f, a region since d-03ca 第 3 段): three eyes side by side
   — who waits, what is ready, and how to resume — then the sky of the nodes
   that matter and the pulse. It draws inside #main from the state the root
   read; the words come from the ui it is handed. */
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

  let data = null;
  let actors = [];
  let t = key => key;
  let lang = 'en';
  /* The sky is measured a frame later; this drops a draw whose state is gone. */
  let drawn = 0;

  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  /* The label a row shows: the first alias, else the id. */
  const label = row => row.alias || row.id;
  const status = row => t(`st.${row.kind}.${row.status}`);

  const head = (cls, n, name, sub) =>
    `<div class="head"><span class="n" data-testid="count">${n}</span><span class="l">${esc(name)}</span><span class="h">${esc(sub)}</span></div>`;

  const rowHtml = (row, st) =>
    `<a class="row" href="#/n/${row.id}"><span class="dot dot-${row.kind}"></span><span class="id">${esc(label(row))}</span><span class="t" title="${esc(row.title)}">${esc(row.title)}</span><span class="st">${st}</span></a>`;

  const questions = () => data.waiting.filter(item => item.waiting === 'question');
  const requirements = () => data.waiting.filter(item => item.waiting === 'requirement');

  /* Nothing has been written yet: the page is not quiet, it has not started
     (n-3e6b). Only the first eye speaks, so the praise words do not claim a
     ledger that is not there. */
  const firstRun = () => !data.seq;

  /* The one word an empty column shows: a bundle chosen by the last write's
     sequence, so a ledger that does not move keeps the same words across
     redraws, and a new write may pick another (n-f3be). */
  function pick(key) {
    const bundle = t(key);
    if (!Array.isArray(bundle)) return bundle;
    const seq = data.resume.last ? data.resume.last.seq : 0;
    return bundle[seq % bundle.length];
  }

  /* One question as a card, as the old page drew it. */
  function card(item) {
    const row = item.row;
    const options = item.options.map(option => `<li>${esc(option)}</li>`).join('');
    const mark = row.unwaited ? ` <i class="unwaited">${esc(t('unwaited'))} · ${esc(window.GyTime.age(row.created, t))}</i>` : '';
    return `<a class="wait" href="#/n/${row.id}"><div class="who">${esc(label(row))} · ${t('decider')} ${esc(item.decider)}${mark}</div><div class="t">${esc(row.title)}</div><ol class="opts">${options}</ol></a>`;
  }

  /* The first eye: the questions that wait on a person, then the filed
     requirements. An empty column is good news, so it says so in one line. */
  function waitColumn() {
    const qs = questions();
    const rs = requirements();
    const sub = `${t('Questions')} ${qs.length} · ${t('Requirements')} ${rs.length}`;
    const eye = head('hot', data.waiting.length, t('eyeWait'), sub);
    if (!data.waiting.length) {
      const word = firstRun() ? t('firstRun') : pick('praiseWait');
      return `<section class="eye hot" data-eye="wait" aria-label="${esc(t('eyeWait'))}">${eye}<div class="nothing" role="status">${esc(word)}</div></section>`;
    }
    const cards = qs.slice(0, 2).map(card).join('');
    const more = qs.length > 2
      ? `<a class="more" href="#/list/Question">${fill(t('moreQuestions'), { n: qs.length - 2 })}</a>`
      : '';
    const rows = rs.length
      ? `<div class="rows">${rs.map(item => rowHtml(item.row, status(item.row))).join('')}</div>`
      : '';
    return `<section class="eye hot" data-eye="wait" aria-label="${esc(t('eyeWait'))}">${eye}${cards}${more}${rows}</section>`;
  }

  /* The second eye: the needs ready to work, with what is left to meet. When
     none is ready, an empty ledger is good news and a full one is a jam: the
     needs still in progress are what is stuck, and only they get the count. */
  function readyColumn() {
    const rows = data.ready
      .map(item => rowHtml(item.row, fill(t('remaining'), { n: item.targets - item.satisfied, m: item.targets })))
      .join('');
    const body = data.ready.length
      ? `<div class="rows">${rows}</div>`
      : firstRun()
        ? ''
        : data.in_progress.length
          ? `<div class="nothing calm" role="status">${esc(pick('blockedNext'))}</div>`
          : `<div class="nothing" role="status">${esc(pick('praiseQuiet'))}</div>`;
    const tail = data.in_progress.length
      ? `<a class="more" href="#/list/Need">${fill(t('allNeeds'), { n: data.in_progress.length })}</a>`
      : '';
    return `<section class="eye next" data-eye="next" aria-label="${esc(t('eyeNext'))}">${head('next', data.ready.length, t('readyHead'), t('readySub'))}${body}${tail}</section>`;
  }

  /* The third eye: the in-progress requirements, then the counts. The outward
     reference is a span, not a link: the row itself is a link to the node page
     (a nested anchor would break the grid). */
  function resumeColumn() {
    const rows = data.resume.in_progress.map(item => {
      const tail = referenceTail(item.reference);
      const ref = tail ? ` · <span class="ref">${esc(tail)}</span>` : '';
      return rowHtml({ id: item.id, kind: 'Requirement', title: item.title, alias: null }, esc(t(`st.Requirement.${item.state}`)) + ref);
    }).join('');
    const last = data.resume.last;
    const lastText = last ? `${window.GyTime.time(last.at, lang)} ${esc(last.actor)} · seq ${last.seq}` : t('none');
    const kv = `<div class="kv"><span>${t('openQuestions')}</span><b>${data.resume.open_questions}</b><span>${t('warnings')}</span><b>${data.resume.warnings || t('none')}</b><span>${t('lastWrite')}</span><b>${lastText}</b>${syncParts()}</div>`;
    const body = data.resume.in_progress.length
      ? `<div class="rows">${rows}</div>`
      : firstRun()
        ? ''
        : `<div class="nothing" role="status">${esc(pick('praiseResume'))}</div>`;
    return `<section class="eye prog" data-eye="resume" aria-label="${esc(t('eyeResume'))}">${head('prog', data.resume.in_progress.length, t('resumeHead'), t('resumeSub'))}${body}${kv}</section>`;
  }

  /* The shared copy's sync as one kv pair per fact, like the other pairs
     (n-94bb). The last line of a failure is cut short; --json keeps it whole. */
  function syncParts() {
    const sync = data.resume.sync;
    if (!sync) return '';
    let out = '';
    if (sync.last_ok_at && sync.last_ok_seq != null) {
      let value = `${window.GyTime.time(sync.last_ok_at, lang)} (seq ${sync.last_ok_seq})`;
      if (sync.pending) value += ` · ${sync.pending} ${t('notPushed')}`;
      out += `<span>${t('lastSynced')}</span><b>${value}</b>`;
    }
    if (sync.last_error) {
      const first = String(sync.last_error).split('\n')[0].slice(0, 80);
      out += `<span>${t('syncFailed')}</span><b>${esc(first)}…</b>`;
    }
    return out;
  }

  /* The number a requirement's outward reference ends with, as a link's word. */
  function referenceTail(reference) {
    if (!reference) return '';
    return String(reference).split('/').filter(Boolean).pop() || '';
  }

  const legend = () =>
    `<span><i style="background:var(--requirement)"></i>${t('skyLegendWait')}</span><span><i style="background:var(--need)"></i>${t('skyLegendNext')}</span><span><i style="background:var(--decision)"></i>${t('skyLegendResume')}</span>`;

  /* The sky, drawn by the helper. The canvas is measured after a frame, so its
     laid-out size is the one the fit uses; a draw whose state is gone drops. */
  async function paintSky(el, state) {
    const token = ++drawn;
    await new Promise(resolve => requestAnimationFrame(resolve));
    if (token !== drawn) return;
    window.GySky.draw(el.querySelector('#sky'), state);
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
    const actor = esc(entry.who || entry.actor);
    const doing = esc(verb(entry));
    const node = entry.node ? `<a href="#/n/${entry.node}">${esc(entry.node)}</a>` : '';
    if (!node) return `${actor} ${doing}`;
    return lang === 'ja' ? `${actor} が ${node} ${doing}` : `${actor} ${doing} ${node}`;
  }

  function line(entry) {
    return `<div class="when">${window.GyTime.time(entry.at, lang)} · seq ${entry.seq} · ${esc(entry.who || entry.actor)}</div><div class="what">${what(entry)}</div><div class="src" title="${esc(entry.source)}">${esc(entry.source)}</div>`;
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

  /* The pulse, in two columns. With no writes there is no "up to" time: the
     line the empty rail and history use says it instead (n-3e6b). */
  function pulse() {
    const rows = data.recent
      .map((entry, index) => `<div class="pi ${actorCls(entry.who || entry.actor)}${index === 0 ? ' new' : ''}" role="listitem">${line(entry)}</div>`)
      .join('');
    const sub = data.recent.length
      ? fill(t('pulseSub'), { n: data.recent.length, t: window.GyTime.time(data.recent[0].at, lang) })
      : t('histEmpty');
    return `<section class="pulse2"><h2>${t('pulse')}</h2><div class="sub">${sub}</div><div class="pl2" role="list" aria-label="${esc(t('pulse'))}">${rows}</div></section>`;
  }

  /* The region: #main only, from the state the root read. */
  function render(state, el, ui) {
    if (!el) return;
    t = ui.t;
    lang = state.lang;
    data = state.now;
    actors = [];
    if (!data) {
      el.innerHTML = '';
      return;
    }
    window.GySync = (data.resume && data.resume.sync) || null;
    const errors = (data.resume && data.resume.errors) || [];
    const errorBlock = errors.length
      ? `<div class="errors" role="status">${errors.map(line => `<div class="err">${esc(line)}</div>`).join('')}</div>`
      : '';
    el.innerHTML =
      `<div class="eyes">${waitColumn()}${readyColumn()}${resumeColumn()}</div>` +
      errorBlock +
      `<div class="sky"><canvas id="sky"></canvas><div class="cap">${t('skyCap')}</div><div class="leg">${legend()}</div></div>` +
      pulse();
    void paintSky(el, state);
  }

  window.GyNow = { render };
})();