/* The top bar region (d-03ca, 第 2 段): the breadcrumb. The node page hands it
   the kind and the scope through state.page (第 3 段); a node that is not there
   still shows its id. The first screen is progress (d-e239), so the crumb's
   root is that screen. */
(function () {
  const EYE_WORD = { wait: 'eyeWait', next: 'readyHead', resume: 'resumeHead' };
  const esc = text => String(text ?? '').replace(/[&<>"]/g, ch => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[ch]));

  function render(state, el, ui) {
    if (!el) return;
    const route = state.route;
    const root = `<a href="#/">${ui.t('tabOverview')}</a><span>›</span>`;
    if (route.name === 'overview' || route.name === 'missing') el.innerHTML = `<span>${ui.t('tabOverview')}</span>`;
    else if (route.name === 'now') el.innerHTML = `${root}<span>${ui.t('now')}</span>`;
    else if (route.name === 'cross') el.innerHTML = `${root}<span>${ui.t('tabCross')}</span>`;
    else if (route.name === 'list') el.innerHTML = `${root}<span>${ui.plural(route.arg)}</span>`;
    else if (route.name === 'eye') el.innerHTML = `${root}<span>${ui.t(EYE_WORD[route.arg])}</span>`;
    else if (route.name === 'history') el.innerHTML = `${root}<span>${ui.t('history')}</span>`;
    else if (route.name === 'node') el.innerHTML = `${root}${trail(state, ui)}`;
  }

  /* The node's trail: this kind's list and the scope, from the page's answer. */
  function trail(state, ui) {
    const node = state.page;
    if (!node) return `<span>${esc(state.route.arg)}</span>`;
    return `<a href="#/list/${node.kind}">${ui.plural(node.kind)}</a><span>›</span>${ui.scopeTag(node.scope)}`;
  }

  window.GyTopbar = { render };
})();
