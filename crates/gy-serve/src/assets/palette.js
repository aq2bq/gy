/* The palette region (d-03ca, 第 2 段; was n-5ca6): the search overlay (#pal),
   opened from the sidebar's one entry (n-d0cc). Keys arrive as intents from
   the root. */
(function () {
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));

  function render(state, el, ui) {
    if (!el) return;
    el.classList.toggle('on', state.palette.open);
    const query = el.querySelector('#palq');
    if (query.value !== state.palette.query) query.value = state.palette.query;
    query.placeholder = ui.t('palPh');
    el.querySelector('#palhint').textContent = ui.t('palHint');
    el.querySelector('#palres').innerHTML = results(state, ui);
  }

  function results(state, ui) {
    const hits = state.search ? state.search.hits || [] : [];
    if (!hits.length) {
      return state.palette.query.trim() ? `<div class="r"><span class="t" style="color:var(--paper-3)">${ui.t('notFound')}</span></div>` : '';
    }
    return hits
      .map((hit, index) => `<div class="r ${index === state.palette.selected ? 'sel' : ''}" role="option" aria-selected="${index === state.palette.selected}" data-act="paletteGo" data-arg="${index}"><span class="dot dot-${hit.kind}"></span><span class="id">${esc(hit.alias || hit.id)}</span><span class="t">${esc(hit.title)}</span>${ui.scopeTag(hit.scope)}</div>`)
      .join('');
  }

  /* The palette's keys as intents; the root turns them into runs. No listeners
     here: the root holds the one keydown (d-03ca). */
  function key(state, event) {
    const hits = state.search ? state.search.hits || [] : [];
    if (event.key === 'ArrowDown') return { type: 'paletteMove', value: Math.min(hits.length - 1, state.palette.selected + 1), prevent: true };
    if (event.key === 'ArrowUp') return { type: 'paletteMove', value: Math.max(0, state.palette.selected - 1), prevent: true };
    if (event.key === 'Escape') return { type: 'paletteClose' };
    if (event.key === 'Enter') return { type: 'paletteEnter' };
    return null;
  }

  window.GyPalette = { render, key };
})();