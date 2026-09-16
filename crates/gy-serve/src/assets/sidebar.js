/* The sidebar region (d-03ca, 第 2 段): the three boxes, the nav, the language
   switch, and the clock. It draws inside the element it is given, and reads
   only the state it is handed. */
(function () {
  const ENTRIES = [['#/', 'now', '◉'], ['#/graph', 'graph', '✦'], ['#/history', 'history', '≡']];
  const KINDS = ['Need', 'Question', 'Decision', 'Requirement', 'Criterion'];
  const EYE_COLOR = { wait: 'requirement', next: 'need', resume: 'decision' };
  const EYE_WORD = { wait: 'eyeWait', next: 'eyeNext', resume: 'eyeResume' };

  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));
  const fill = (text, values) => text.replace(/\{(\w+)\}/g, (all, key) => (key in values ? values[key] : all));

  function stamp(at) {
    return new Date(at).toLocaleString(window.document.documentElement.lang === 'ja' ? 'ja-JP' : 'en-GB', { month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit' });
  }

  function counter(shell, kind) {
    const row = (shell.kinds || []).find(item => item.kind === kind);
    if (!row) return '<span class="cnt"></span>';
    return kind === 'Decision'
      ? `<span class="cnt"><b>${row.total}</b></span>`
      : `<span class="cnt"><b>${row.open}</b> / ${row.total}</span>`;
  }

  function drawEyes(state, el, ui) {
    const now = state.now;
    const n = key => (now ? now[key].length : 0);
    const item = (key, count) =>
      `<a href="#/eye/${key}" style="--ec:var(--${EYE_COLOR[key]})"><b>${count}</b><small>${ui.t(EYE_WORD[key])}</small></a>`;
    el.querySelector('#eyes3').innerHTML =
      item('wait', now ? now.waiting.length : 0) +
      item('next', now ? now.ready.length : 0) +
      item('resume', now ? now.resume.in_progress.length : 0);
  }

  function drawNav(state, el, ui) {
    const shell = state.shell || { seq: 0, scopes: [], kinds: [] };
    const route = state.route;
    const entry = ([href, key, mark]) => {
      const on = route.name === key;
      return `<a href="${href}" class="${on ? 'on' : ''}">${mark} ${ui.t(key)}</a>`;
    };
    const kind = name => {
      const on = route.name === 'list' && route.arg === name;
      return `<a href="#/list/${name}" class="${on ? 'on' : ''}"><span class="dot dot-${name}"></span>${ui.plural(name)}${counter(shell, name)}</a>`;
    };
    const scopeRow = item =>
      `<button data-s="${esc(item.name)}" data-act="setScope" data-arg="${esc(item.name)}" class="${state.scope === item.name ? 'on' : ''}">${ui.scopeTag(item.name)}<span class="cnt">${item.count}</span></button>`;
    const nodes = (shell.scopes || []).reduce((sum, item) => sum + item.count, 0);
    el.querySelector('#nav').innerHTML =
      ENTRIES.map(entry).join('') +
      `<div class="h">${ui.t('nodes')}</div>` +
      KINDS.map(kind).join('') +
      `<div class="h">${ui.t('scopes')}</div>` +
      `<button data-s="all" data-act="setScope" data-arg="all" class="${state.scope === 'all' ? 'on' : ''}">${ui.t('all')}<span class="cnt">${nodes}</span></button>` +
      shell.scopes.map(scopeRow).join('');
  }

  function drawClock(state, el, ui) {
    const clock = el.querySelector('#clock');
    const shell = state.shell;
    const seq = shell ? shell.seq : 0;
    if (state.at === null) {
      const when = seq > 0 ? `<br>${stamp(shell.at)}` : '';
      clock.innerHTML = `<span class="live${state.live ? '' : ' off'}"></span>${ui.t('canon')} <b>${seq}</b>${when}`;
      return;
    }
    const tick = ui.tickAt(state.at);
    const time = tick ? ui.when(tick.at) : ui.t('start');
    clock.innerHTML = fill(ui.t('viewing'), { t: time });
  }

  function render(state, el, ui) {
    if (!el) return;
    el.querySelectorAll('#lang button').forEach(button => {
      button.classList.toggle('on', button.dataset.l === state.lang);
    });
    const logo = el.querySelector('.wordmark .logo');
    if (logo) logo.classList.toggle('off', !state.live);
    drawEyes(state, el, ui);
    drawNav(state, el, ui);
    drawClock(state, el, ui);
  }

  window.GySidebar = { render };
})();