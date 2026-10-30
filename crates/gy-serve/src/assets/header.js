/* The header region: the three screens' tabs, the jump box, the log's position,
   the connection mark, the language switch, and the repository link. It draws
   inside `.top` and only changes its own slots, so the jump box keeps its text
   while the page redraws. */
(function () {
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));

  /* The three tabs, in the order the work moves (d-647f): progress, across,
     detail. The detail tab needs a node this tab has opened; before one, it is
     a plain mark. */
  function tabs(state, ui) {
    const route = state.route;
    const last = state.trail[state.trail.length - 1];
    const tab = (href, key, on, note) => {
      const inner = `<span>${esc(ui.t(key))}</span>${note ? `<span class="n">${esc(note)}</span>` : ''}`;
      return href ? `<a href="${href}" class="${on ? 'on' : ''}">${inner}</a>` : `<span class="off">${inner}</span>`;
    };
    return tab('#/', 'tabOverview', route.name === 'overview', null) +
      `<span class="arrow">→</span>` +
      tab('#/cross', 'tabCross', route.name === 'cross', null) +
      `<span class="arrow">→</span>` +
      tab(last ? `#/n/${encodeURIComponent(last)}` : null, 'tabDetail', route.name === 'node', last || null);
  }

  /* The log's point: the head at first, the point and the head after a rewind.
     The band knows the log's end, so its max is the denominator (n-32a9). */
  function position(state) {
    const shell = state.shell;
    const at = state.at;
    if (!shell) return '';
    if (at === null) return `seq ${shell.seq}`;
    const head = state.band ? state.band.max : shell.seq;
    return `seq ${at} / ${head}`;
  }

  function mark(state, el, ui) {
    const live = el.querySelector('#live');
    if (!live) return;
    live.classList.toggle('off', !state.live);
    live.setAttribute('title', ui.t(state.live ? 'liveOn' : 'liveOff'));
  }

  function language(state, el) {
    el.querySelectorAll('#lang button').forEach(button => {
      button.classList.toggle('on', button.dataset.l === state.lang);
    });
  }

  function repository(el, ui) {
    const repo = el.querySelector('[data-testid="repo"]');
    if (!repo) return;
    const name = ui.t('repoLabel');
    repo.setAttribute('aria-label', name);
    repo.setAttribute('title', name);
  }

  /* An id (or alias) opens the node; anything else is a word across. */
  function jump(el, ui) {
    const box = el.querySelector('#jump');
    if (!box) return;
    box.placeholder = ui.t('jumpPh');
    box.setAttribute('aria-label', ui.t('jumpPh'));
    box.onkeydown = event => {
      if (event.key !== 'Enter') return;
      const value = box.value.trim();
      if (!value) return;
      box.value = '';
      ui.navigate(/^[a-z]+-[0-9a-z]+$/i.test(value) ? `#/n/${encodeURIComponent(value)}` : `#/cross?q=${encodeURIComponent(value)}`);
    };
  }

  function render(state, el, ui) {
    if (!el) return;
    const box = el.querySelector('#tabs');
    if (box) box.innerHTML = tabs(state, ui);
    const seq = el.querySelector('#seq');
    if (seq) seq.textContent = position(state);
    mark(state, el, ui);
    language(state, el);
    repository(el, ui);
    jump(el, ui);
  }

  window.GyHeader = { render };
})();
